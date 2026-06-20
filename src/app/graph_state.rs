use crate::citation::graph::{CitationGraph, GraphLayout, RenderMode};

#[derive(Clone, Debug)]
pub struct GraphState {
    pub doc_ids: Vec<i64>,
    pub graph: CitationGraph,
    pub layout: Option<GraphLayout>,
    pub focused_node: Option<usize>,
    pub render_mode: RenderMode,
    pub cache_hit: bool,
}

impl GraphState {
    pub fn new(graph: CitationGraph, cache_hit: bool) -> Self {
        let node_count = graph.node_count();
        let render_mode = RenderMode::for_node_count(node_count);
        let doc_ids: Vec<i64> = graph
            .doc_to_node
            .keys()
            .copied()
            .collect();
        GraphState {
            doc_ids,
            graph,
            layout: None,
            focused_node: if node_count > 0 { Some(0) } else { None },
            render_mode,
            cache_hit,
        }
    }

    pub fn cycle_render_mode(&mut self) {
        self.render_mode = match self.render_mode {
            RenderMode::Visual => RenderMode::Table,
            RenderMode::Table => RenderMode::Visual,
        };
    }

    pub fn focus_next(&mut self, step: isize) {
        let count = self.graph.node_count();
        if count == 0 {
            self.focused_node = None;
            return;
        }
        let current = self.focused_node.unwrap_or(0) as isize;
        let next = ((current + step).rem_euclid(count as isize)) as usize;
        self.focused_node = Some(next);
    }
}
