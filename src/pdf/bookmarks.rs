use anyhow::{Result, anyhow};
use lopdf::Document as LopdfDocument;
use lopdf::Object;
use std::collections::HashMap;
use std::path::Path;

/// Extract PDF bookmarks/outline as (title, page_number) pairs.
/// Page numbers are 1-based. Returns empty vec if no outlines exist.
pub fn extract_bookmarks(path: &Path) -> Result<Vec<(String, i64)>> {
    let doc = LopdfDocument::load(path)?;

    // get_pages() returns BTreeMap<u32 (page_num), ObjectId>
    // Build reverse lookup: ObjectId → page number
    let pages = doc.get_pages();
    let page_lookup: HashMap<lopdf::ObjectId, u32> = pages
        .iter()
        .map(|(page_num, obj_id)| (*obj_id, *page_num))
        .collect();

    let root_ref = doc
        .trailer
        .get(b"Root")
        .map_err(|_| anyhow!("트레일러에 Root 없음"))?
        .as_reference()?;
    let catalog = doc.get_object(root_ref)?.as_dict()?;

    let outlines_ref = match catalog.get(b"Outlines") {
        Ok(obj) => obj.as_reference()?,
        Err(_) => return Ok(Vec::new()),
    };
    let outlines = doc.get_object(outlines_ref)?.as_dict()?;

    let first_ref = match outlines.get(b"First") {
        Ok(obj) => obj.as_reference()?,
        Err(_) => return Ok(Vec::new()),
    };

    let mut result = Vec::new();
    let mut visited = HashMap::new();
    traverse_outline(&doc, first_ref, &page_lookup, &mut result, 0, &mut visited)?;
    Ok(result)
}

fn traverse_outline(
    doc: &LopdfDocument,
    item_ref: lopdf::ObjectId,
    pages: &HashMap<lopdf::ObjectId, u32>,
    result: &mut Vec<(String, i64)>,
    depth: u32,
    visited: &mut HashMap<lopdf::ObjectId, ()>,
) -> Result<()> {
    let mut current_ref = Some(item_ref);

    while let Some(ref_id) = current_ref {
        if visited.contains_key(&ref_id) {
            break;
        }
        visited.insert(ref_id, ());

        let item = doc.get_object(ref_id)?.as_dict()?;

        let title = item
            .get(b"Title")
            .and_then(|o| o.as_str())
            .unwrap_or(b"")
            .to_vec();
        let title = String::from_utf8_lossy(&title).to_string();

        let page_num = get_dest_page(doc, item, pages).unwrap_or(0);

        if !title.is_empty() {
            let prefix = "  ".repeat(depth as usize);
            result.push((format!("{}{}", prefix, title), page_num as i64));
        }

        if let Ok(first_child_ref) = item.get(b"First").and_then(|o| o.as_reference()) {
            traverse_outline(doc, first_child_ref, pages, result, depth + 1, visited)?;
        }

        current_ref = item.get(b"Next").ok().and_then(|o| o.as_reference().ok());
    }

    Ok(())
}

fn get_dest_page(
    doc: &LopdfDocument,
    item: &lopdf::Dictionary,
    pages: &HashMap<lopdf::ObjectId, u32>,
) -> Option<u32> {
    if let Ok(dest) = item.get(b"Dest")
        && let Some(page) = resolve_dest_page(dest, pages)
    {
        return Some(page);
    }

    if let Ok(action_ref) = item.get(b"A").and_then(|o| o.as_reference())
        && let Ok(action) = doc.get_object(action_ref).and_then(|o| o.as_dict())
        && let Ok(d) = action.get(b"D")
        && let Some(page) = resolve_dest_page(d, pages)
    {
        return Some(page);
    }
    None
}

fn resolve_dest_page(dest: &Object, pages: &HashMap<lopdf::ObjectId, u32>) -> Option<u32> {
    match dest {
        Object::Array(arr) => {
            if let Some(Object::Reference(id)) = arr.first() {
                return pages.get(id).copied();
            }
        }
        Object::Reference(id) => {
            return pages.get(id).copied();
        }
        _ => {}
    }
    None
}
