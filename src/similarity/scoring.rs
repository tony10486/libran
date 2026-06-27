use std::collections::HashMap;

use super::config::SimilarityConfig;

/// Result of computing similarity between a reference paper and another paper.
#[derive(Debug, Clone)]
pub struct DocumentScore {
    pub document_id: i64,
    pub total_score: f64,
    pub udc_score: f64,
    pub tag_score: f64,
    pub citation_score: f64,
    pub year_score: f64,
    pub conference_score: f64,
}

/// All data needed about a single document to compute scores.
#[derive(Debug, Clone)]
pub struct DocumentFeatures {
    pub id: i64,
    pub udc_notations: Vec<String>,
    pub tags: Vec<String>,
    pub cited_docs: Vec<i64>,    // documents this paper cites
    pub cited_by_docs: Vec<i64>, // documents that cite this paper
    pub pub_year: Option<i64>,
    pub conference: Option<String>,
}

/// In-memory UDC tree for fast ancestor-path resolution.
pub struct UdcTree {
    /// notation -> parent_notation
    parents: HashMap<String, String>,
    /// Precomputed depth for each known notation.
    depths: HashMap<String, usize>,
}

impl UdcTree {
    pub fn new(node_parents: HashMap<String, String>) -> Self {
        let mut depths = HashMap::new();
        for notation in node_parents.keys() {
            Self::compute_depth(notation, &node_parents, &mut depths);
        }
        UdcTree {
            parents: node_parents,
            depths,
        }
    }

    fn compute_depth(
        notation: &str,
        parents: &HashMap<String, String>,
        cache: &mut HashMap<String, usize>,
    ) -> usize {
        if let Some(&d) = cache.get(notation) {
            return d;
        }
        let depth = if let Some(parent) = parents.get(notation) {
            Self::compute_depth(parent, parents, cache) + 1
        } else {
            0 // root
        };
        cache.insert(notation.to_string(), depth);
        depth
    }

    /// Build the full ancestor chain for a notation (including itself).
    /// Returns [root, ..., parent, notation].
    fn ancestor_path(&self, notation: &str) -> Vec<String> {
        let mut path = vec![notation.to_string()];
        let mut current = notation;
        while let Some(parent) = self.parents.get(current) {
            path.push(parent.clone());
            current = parent;
        }
        path.reverse(); // root first
        path
    }

    /// Depth at which the two notations share their lowest common ancestor.
    /// Returns the depth of the LCA in the tree (0 = root).
    /// Returns None if no common ancestor found (shouldn't happen in a connected tree).
    pub fn lca_depth(&self, a: &str, b: &str) -> Option<usize> {
        let path_a = self.ancestor_path(a);
        let path_b = self.ancestor_path(b);
        let mut lca_depth = 0;
        let max_len = path_a.len().min(path_b.len());
        for i in 0..max_len {
            if path_a[i] == path_b[i] {
                lca_depth = i;
            } else {
                break;
            }
        }
        Some(lca_depth)
    }

    /// Compute UDC score between two sets of notations.
    /// Returns (score, top_level_match_found) where top_level_match_found indicates
    /// that at least the broadest category matches.
    pub fn score_between(
        &self,
        notations_a: &[String],
        notations_b: &[String],
        config: &SimilarityConfig,
    ) -> (f64, bool) {
        if notations_a.is_empty() || notations_b.is_empty() {
            return (0.0, false);
        }

        let mut best_score = 0.0_f64;
        let mut top_level_match = false;

        for na in notations_a {
            let depth_a = self.depths.get(na.as_str()).copied().unwrap_or(0);
            for nb in notations_b {
                let depth_b = self.depths.get(nb.as_str()).copied().unwrap_or(0);
                if na == nb {
                    // Exact same notation — leaf match
                    best_score = best_score.max(config.udc_leaf_match);
                    top_level_match = true;
                    continue;
                }
                let Some(lca_d) = self.lca_depth(na, nb) else {
                    continue;
                };

                // Determine "levels up" for each paper: how far up we went to find commonality
                let levels_up_a = depth_a.saturating_sub(lca_d);
                let levels_up_b = depth_b.saturating_sub(lca_d);
                let max_levels_up = levels_up_a.max(levels_up_b);

                if max_levels_up == 0 {
                    // Same exact notation — leaf match
                    best_score = best_score.max(config.udc_leaf_match);
                    top_level_match = true;
                } else if max_levels_up == 1 {
                    // Parent level match (e.g., both under same parent)
                    best_score = best_score.max(config.udc_parent_match);
                    top_level_match = true;
                } else if lca_d > 0 {
                    // They share at least a top-level subject domain.
                    // max_levels_up >= 2 → grandparent match minimum
                    best_score = best_score.max(config.udc_grandparent_match);
                    top_level_match = true;
                }
                // If lca_d == 0 (only root/virtual root matches),
                // top_level_match stays false → paper is excluded unless override
            }
        }
        (best_score, top_level_match)
    }

    /// Get the depth of a specific notation.
    #[allow(dead_code)]
    pub fn depth_of(&self, notation: &str) -> Option<usize> {
        self.depths.get(notation).copied()
    }
}

/// Parse a UDC notation string into individual component notations.
/// Handles compound codes separated by `:` and strips auxiliary modifiers
/// to extract the base notation.
///
/// Examples:
/// - "517.9" → ["517.9"]
/// - "51-72:538.91" → ["51", "538.91"]  (strip -72 auxiliary)
/// - "517.9:538.91" → ["517.9", "538.91"]
pub fn parse_udc_notation(raw: &str) -> Vec<String> {
    let mut notations = Vec::new();
    // Split on colon to get compound parts
    for part in raw.split(':') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        // Extract base notation before auxiliary hyphens, slashes, etc.
        // UDC base is the number before '-', '/', '(' etc.
        let base = part
            .split(&['-', '/', '(', ')', '=', '+'][..])
            .next()
            .unwrap_or(part)
            .trim()
            .to_string();
        if !base.is_empty() && base.chars().any(|c| c.is_ascii_digit()) {
            notations.push(base);
        }
    }
    // Also add the raw notation itself so compound codes can match directly
    if !notations.contains(&raw.to_string()) && !raw.is_empty() {
        notations.push(raw.to_string());
    }
    notations
}

/// Compute similarity scores for ALL documents against one reference document.
/// Returns a Vec of (DocumentScore) sorted by descending total_score.
pub fn compute_scores(
    reference: &DocumentFeatures,
    candidates: &[DocumentFeatures],
    udc_tree: &UdcTree,
    config: &SimilarityConfig,
) -> Vec<DocumentScore> {
    let mut scores: Vec<DocumentScore> = Vec::with_capacity(candidates.len());

    // Precompute reference tags as a set for fast lookup
    let ref_tags: std::collections::HashSet<&str> =
        reference.tags.iter().map(|s| s.as_str()).collect();
    let ref_id = reference.id;
    let ref_year = reference.pub_year;
    let ref_conference = reference.conference.as_deref();

    for doc in candidates {
        if doc.id == ref_id {
            continue; // skip self
        }

        // 1. UDC score + top-level check
        let (udc_score, top_level_match) =
            udc_tree.score_between(&reference.udc_notations, &doc.udc_notations, config);

        // 2. Tag score
        let tag_score = if !ref_tags.is_empty() && !doc.tags.is_empty() {
            let matching_tags = doc
                .tags
                .iter()
                .filter(|t| ref_tags.contains(t.as_str()))
                .count();
            if matching_tags > 0 {
                config.tag_match * matching_tags as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        // 3. Citation score
        let citation_score = compute_citation_score(reference, doc, config);

        // 4. Year proximity score
        let year_score = compute_year_score(ref_year, doc.pub_year, config);

        // 5. Conference score
        let conference_score =
            compute_conference_score(ref_conference, doc.conference.as_deref(), config);

        // Determine if this paper should be excluded (UDC top-level mismatch)
        // Exclusion rule: if no UDC top-level match AND no tag match AND no citation match
        let has_override = tag_score > 0.0 || citation_score > 0.0;

        let total_score = if !top_level_match && !has_override {
            // Exclude this paper from similarity results
            0.0
        } else {
            udc_score + tag_score + citation_score + year_score + conference_score
        };

        scores.push(DocumentScore {
            document_id: doc.id,
            total_score,
            udc_score,
            tag_score,
            citation_score,
            year_score,
            conference_score,
        });
    }

    // Sort by total_score descending
    scores.sort_by(|a, b| {
        b.total_score
            .partial_cmp(&a.total_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    scores
}

fn compute_citation_score(
    reference: &DocumentFeatures,
    doc: &DocumentFeatures,
    config: &SimilarityConfig,
) -> f64 {
    let ref_id = reference.id;
    let doc_id = doc.id;

    // Mutual citation: doc cites ref AND ref cites doc
    let doc_cites_ref = doc.cited_docs.contains(&ref_id);
    let ref_cites_doc = reference.cited_docs.contains(&doc_id);
    if doc_cites_ref && ref_cites_doc {
        return config.mutual_citation;
    }

    // doc is cited by reference: doc appears in reference's cited_docs
    // Actually "cited by reference paper" means this document IS CITED BY the reference.
    // Equivalent to: reference.cited_docs contains doc.id
    if reference.cited_docs.contains(&doc_id) {
        return config.cited_by;
    }

    // Also check if doc cites reference (the reference is cited by doc)
    // This is "mutual" only if both, but we can also count single-direction.
    // The user's spec mentions: "기준 논문에서 피인용됨" = "cited by the reference paper" (weight 10)
    // and "기존 논문과 상호인용됨" = "mutual citation with the existing paper" (weight 20)
    // So we only count:
    // - reference cites doc → cited_by (weight 10)
    // - mutual → mutual_citation (weight 20)
    // - doc cites reference → no direct score (this makes the relationship directional)

    0.0
}

fn compute_year_score(
    ref_year: Option<i64>,
    doc_year: Option<i64>,
    config: &SimilarityConfig,
) -> f64 {
    match (ref_year, doc_year) {
        (Some(ry), Some(dy)) => {
            let diff = (ry - dy).unsigned_abs() as f64;
            let score = 1.0 - (diff / config.year_proximity_scale).min(1.0);
            score.max(0.0) * config.year_proximity_max
        }
        _ => 0.0,
    }
}

fn compute_conference_score(
    ref_conf: Option<&str>,
    doc_conf: Option<&str>,
    config: &SimilarityConfig,
) -> f64 {
    match (ref_conf, doc_conf) {
        (Some(rc), Some(dc)) if rc == dc => config.same_conference,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tree() -> UdcTree {
        let mut parents = HashMap::new();
        // UDC hierarchy
        parents.insert("5".to_string(), String::new()); // root children have empty parent
        parents.insert("51".to_string(), "5".to_string());
        parents.insert("512".to_string(), "51".to_string());
        parents.insert("514".to_string(), "51".to_string());
        parents.insert("517".to_string(), "51".to_string());
        parents.insert("517.9".to_string(), "517".to_string());
        parents.insert("53".to_string(), "5".to_string());
        parents.insert("531".to_string(), "53".to_string());
        parents.insert("6".to_string(), String::new());
        parents.insert("61".to_string(), "6".to_string());
        UdcTree::new(parents)
    }

    #[test]
    fn test_lca_same_notation() {
        let tree = make_tree();
        let d = tree.lca_depth("517.9", "517.9").unwrap();
        assert_eq!(d, 4); // root(0)->5(1)->51(2)->517(3)->517.9(4) => depth 4
    }

    #[test]
    fn test_lca_siblings() {
        let tree = make_tree();
        // 512 and 514 both under 51
        let d = tree.lca_depth("512", "514").unwrap();
        assert_eq!(d, 2); // root->5->51 => depth 2
    }

    #[test]
    fn test_lca_different_toplevel() {
        let tree = make_tree();
        let d = tree.lca_depth("512", "61").unwrap();
        assert_eq!(d, 0); // only root matches
    }

    #[test]
    fn test_parse_compound() {
        let notations = parse_udc_notation("51-72:517.9");
        assert!(notations.contains(&"51".to_string()));
        assert!(notations.contains(&"517.9".to_string()));
    }

    #[test]
    fn test_parse_simple() {
        let notations = parse_udc_notation("517.9");
        assert_eq!(notations.len(), 1);
        assert_eq!(notations[0], "517.9");
    }

    #[test]
    fn test_score_leaf_match() {
        let tree = make_tree();
        let config = SimilarityConfig::default();
        let (score, matched) =
            tree.score_between(&["517.9".to_string()], &["517.9".to_string()], &config);
        assert!(matched);
        assert_eq!(score, config.udc_leaf_match);
    }

    #[test]
    fn test_score_parent_match() {
        let tree = make_tree();
        let config = SimilarityConfig::default();
        // 512 (Algebra) and 514 (Geometry) both under 51 (Mathematics)
        let (score, matched) =
            tree.score_between(&["512".to_string()], &["514".to_string()], &config);
        assert!(matched);
        assert_eq!(score, config.udc_parent_match);
    }

    #[test]
    fn test_score_grandparent_match() {
        let tree = make_tree();
        let config = SimilarityConfig::default();
        // 517.9 (depth 4) and 512 (depth 3) - LCA at depth 2 (51)
        // levels_up: 4-2=2, 3-2=1, max=2 -> grandparent match
        let (score, matched) =
            tree.score_between(&["517.9".to_string()], &["512".to_string()], &config);
        assert!(matched);
        assert_eq!(score, config.udc_grandparent_match);
    }

    #[test]
    fn test_score_no_toplevel_match() {
        let tree = make_tree();
        let config = SimilarityConfig::default();
        // 512 (Math) and 61 (Medicine) -> different top-level
        let (score, matched) =
            tree.score_between(&["512".to_string()], &["61".to_string()], &config);
        assert!(!matched);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_score_broad_vs_deep_same_toplevel() {
        let tree = make_tree();
        let config = SimilarityConfig::default();
        // "5" (broad Math/Science) and "517.9" (very specific differential equations)
        // LCA at depth 1 ("5"), levels_up: 0 and 3 → max=3, but lca_d=1>0 → grandparent
        let (score, matched) =
            tree.score_between(&["5".to_string()], &["517.9".to_string()], &config);
        assert!(matched);
        assert_eq!(score, config.udc_grandparent_match);
    }
}
