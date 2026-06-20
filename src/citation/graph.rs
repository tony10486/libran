use anyhow::Result;
use petgraph::algo::kosaraju_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use rusqlite::Connection;

use super::match_refs::MatchStatus;

#[derive(Clone, Debug)]
pub struct GraphNode {
    pub doc_id: i64,
    pub label: String,
    pub citation_count: usize,
}

#[derive(Clone, Debug)]
pub struct GraphEdge {
    pub match_status: MatchStatus,
    pub confidence: f64,
}

pub type CitationDiGraph = DiGraph<GraphNode, GraphEdge>;

#[derive(Clone, Debug)]
pub struct CitationGraph {
    pub inner: CitationDiGraph,
    pub doc_to_node: std::collections::HashMap<i64, NodeIndex>,
}

#[derive(Clone, Debug)]
pub struct GraphLayout {
    pub node_positions: Vec<NodeLayout>,
    pub render_mode: RenderMode,
}

#[derive(Clone, Debug)]
pub struct NodeLayout {
    pub node_idx: usize,
    pub row: u16,
    pub col: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderMode {
    Visual,
    Table,
}

impl CitationGraph {
    pub fn build(conn: &Connection, doc_ids: &[i64]) -> Result<Self> {
        let mut graph: CitationDiGraph = DiGraph::new();
        let mut doc_to_node = std::collections::HashMap::new();

        let id_set: std::collections::HashSet<i64> = doc_ids.iter().copied().collect();

        for &doc_id in doc_ids {
            let label = fetch_citation_key_or_title(conn, doc_id)?;
            let cited_by_count = count_cited_by_in_set(conn, doc_id, &id_set)?;

            let node = GraphNode {
                doc_id,
                label,
                citation_count: cited_by_count,
            };
            let idx = graph.add_node(node);
            doc_to_node.insert(doc_id, idx);
        }

        let mut edges_to_add = Vec::new();
        for &doc_id in doc_ids {
            let cited = fetch_cited_docs_in_set(conn, doc_id, &id_set)?;
            for (target_id, status, confidence) in cited {
                edges_to_add.push((doc_id, target_id, status, confidence));
            }
        }

        for (src_id, tgt_id, status, confidence) in edges_to_add {
            if let (Some(&src_idx), Some(&tgt_idx)) =
                (doc_to_node.get(&src_id), doc_to_node.get(&tgt_id))
            {
                graph.add_edge(
                    src_idx,
                    tgt_idx,
                    GraphEdge {
                        match_status: status,
                        confidence,
                    },
                );
            }
        }

        Ok(CitationGraph {
            inner: graph,
            doc_to_node,
        })
    }

    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }
}

impl RenderMode {
    pub fn for_node_count(count: usize) -> Self {
        if count <= 15 {
            RenderMode::Visual
        } else {
            RenderMode::Table
        }
    }
}

pub fn compute_layout(graph: &CitationGraph) -> GraphLayout {
    let node_count = graph.inner.node_count();
    let render_mode = RenderMode::for_node_count(node_count);

    if render_mode == RenderMode::Table {
        return GraphLayout {
            node_positions: Vec::new(),
            render_mode,
        };
    }

    let sccs = kosaraju_scc(&graph.inner);

    let mut node_scc_id: std::collections::HashMap<NodeIndex, usize> =
        std::collections::HashMap::new();
    for (scc_idx, scc) in sccs.iter().enumerate() {
        for &node_idx in scc {
            node_scc_id.insert(node_idx, scc_idx);
        }
    }

    let condensed = build_condensed_dag(&graph.inner, &sccs, &node_scc_id);
    let condensed_layers = assign_layers(&condensed);

    let mut positions: Vec<NodeLayout> = Vec::new();
    for orig_idx in graph.inner.node_indices() {
        let scc_id = node_scc_id.get(&orig_idx).copied().unwrap_or(0);
        let layer = condensed_layers.get(&scc_id).copied().unwrap_or(0);

        let _nodes_in_layer = graph
            .inner
            .node_indices()
            .filter(|n| {
                let sid = node_scc_id.get(n).copied().unwrap_or(0);
                condensed_layers.get(&sid).copied().unwrap_or(0) == layer
            })
            .count();

        let pos_in_layer = positions
            .iter()
            .filter(|p| p.row as usize == layer)
            .count();

        let col_spacing = 20u16;
        let row_spacing = 3u16;

        positions.push(NodeLayout {
            node_idx: orig_idx.index(),
            row: (layer * (row_spacing as usize + 1)) as u16,
            col: (pos_in_layer * (col_spacing as usize + 1)) as u16,
        });
    }

    GraphLayout {
        node_positions: positions,
        render_mode,
    }
}

type CondensedGraph = DiGraph<usize, ()>;

fn build_condensed_dag(
    original: &CitationDiGraph,
    sccs: &[Vec<NodeIndex>],
    node_scc_id: &std::collections::HashMap<NodeIndex, usize>,
) -> CondensedGraph {
    let mut condensed: CondensedGraph = DiGraph::new();
    for i in 0..sccs.len() {
        condensed.add_node(i);
    }

    for edge in original.edge_references() {
        let src_scc = node_scc_id.get(&edge.source()).copied().unwrap_or(0);
        let tgt_scc = node_scc_id.get(&edge.target()).copied().unwrap_or(0);
        if src_scc != tgt_scc {
            let src_node = condensed
                .node_indices()
                .find(|n| condensed[*n] == src_scc);
            let tgt_node = condensed
                .node_indices()
                .find(|n| condensed[*n] == tgt_scc);
            if let (Some(s), Some(t)) = (src_node, tgt_node) {
                let exists = condensed
                    .edges_connecting(s, t)
                    .any(|e| e.weight() == &());
                if !exists {
                    condensed.add_edge(s, t, ());
                }
            }
        }
    }

    condensed
}

fn assign_layers(condensed: &CondensedGraph) -> std::collections::HashMap<usize, usize> {
    let mut layers: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    let mut sources: Vec<NodeIndex> = condensed
        .node_indices()
        .filter(|n| condensed.neighbors_directed(*n, petgraph::Direction::Incoming).count() == 0)
        .collect();

    if sources.is_empty() && condensed.node_count() > 0 {
        if let Some(first) = condensed.node_indices().next() {
            sources.push(first);
        }
    }

    for src in &sources {
        let scc_id = condensed[*src];
        layers.insert(scc_id, 0);
    }

    let mut queue: std::collections::VecDeque<NodeIndex> = sources.into_iter().collect();
    while let Some(node) = queue.pop_front() {
        let current_layer = layers.get(&condensed[node]).copied().unwrap_or(0);
        for neighbor in condensed.neighbors_directed(node, petgraph::Direction::Outgoing) {
            let neighbor_id = condensed[neighbor];
            let new_layer = current_layer + 1;
            let existing = layers.get(&neighbor_id).copied().unwrap_or(0);
            if new_layer > existing {
                layers.insert(neighbor_id, new_layer);
                queue.push_back(neighbor);
            }
        }
    }

    layers
}

fn fetch_citation_key_or_title(conn: &Connection, doc_id: i64) -> Result<String> {
    let result: Option<String> = conn
        .query_row(
            "SELECT citation_key, title FROM documents WHERE id = ?1",
            rusqlite::params![doc_id],
            |row| {
                let key: Option<String> = row.get(0)?;
                let title: String = row.get(1)?;
                Ok(key.or_else(|| Some(title)))
            },
        )
        .ok()
        .flatten();
    Ok(result.unwrap_or_else(|| format!("doc_{}", doc_id)))
}

fn count_cited_by_in_set(
    conn: &Connection,
    doc_id: i64,
    id_set: &std::collections::HashSet<i64>,
) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT citing_id FROM citation_relations WHERE cited_id = ?1",
    )?;
    let rows = stmt.query_map(rusqlite::params![doc_id], |row| row.get::<_, i64>(0))?;
    let count = rows
        .filter_map(|r| r.ok())
        .filter(|id| id_set.contains(id))
        .count();
    Ok(count)
}

fn fetch_cited_docs_in_set(
    conn: &Connection,
    doc_id: i64,
    id_set: &std::collections::HashSet<i64>,
) -> Result<Vec<(i64, MatchStatus, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT cited_id, match_status, confidence FROM citation_relations WHERE citing_id = ?1",
    )?;
    let rows = stmt.query_map(rusqlite::params![doc_id], |row| {
        let cited_id: i64 = row.get(0)?;
        let status_str: String = row.get(1)?;
        let confidence: f64 = row.get(2)?;
        Ok((cited_id, status_str, confidence))
    })?;

    let mut result = Vec::new();
    for row in rows {
        let (cited_id, status_str, confidence) = match row {
            Ok(r) => r,
            Err(_) => continue,
        };
        if id_set.contains(&cited_id) {
            if let Some(status) = MatchStatus::from_str(&status_str) {
                result.push((cited_id, status, confidence));
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_mode_threshold() {
        assert_eq!(RenderMode::for_node_count(10), RenderMode::Visual);
        assert_eq!(RenderMode::for_node_count(15), RenderMode::Visual);
        assert_eq!(RenderMode::for_node_count(16), RenderMode::Table);
        assert_eq!(RenderMode::for_node_count(100), RenderMode::Table);
    }
}
