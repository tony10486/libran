use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::citation::generate_citation_key;
use crate::citation::cache;
use crate::citation::entry;
use crate::citation::extract;
use crate::citation::graph;
use crate::citation::graph::CitationGraph;
use crate::citation::match_refs;
use crate::db::documents;
use crate::export::{export, ExportFormat};
use crate::pdf;
use crate::similarity::scoring::{compute_scores, DocumentFeatures};
use crate::storage::library;

use super::action::{AppAction, GraphDirection};
use super::graph_state::GraphState;
use super::state::PanelFocus;
use super::AppState;

const EDIT_FIELDS: &[&str] = &["제목", "저자", "저널", "학회", "연도", "DOI", "arXiv", "초록", "키워드"];

pub fn handle_action(state: &mut AppState, action: AppAction) -> bool {
    match action {
        AppAction::SystemShutdown => return true,
        AppAction::KeyPressed(key) => return handle_key(state, key),
        AppAction::DragDetected(path) => handle_drag(state, path),
        AppAction::MetadataExtracted(meta, original_path) => handle_metadata(state, meta, original_path),
        AppAction::MetadataSaved(id) => {
            if state.api_mode.allows_api_calls() {
                let doc_opt = {
                    if let Ok(conn) = state.db.lock() {
                        documents::get_by_id(&conn, id).ok().flatten()
                    } else {
                        None
                    }
                };
                if let Some(doc) = doc_opt {
                    try_api_lookup(state.action_tx.clone(), state.api_mode.clone(), &doc);
                }
            }
            state.reload_documents();
            state.finish_processing("문헌 추가 완료");
        }
        AppAction::ApiLookupSuccess(meta, doc_id) => apply_api_metadata(state, meta, doc_id),
        AppAction::ApiLookupFailed(msg) => state.set_status(&format!("API 실패: {}", msg)),
        AppAction::ApiLookupSkipped(msg) => state.set_status(&msg),
        AppAction::ToggleApiMode => {
            state.cycle_api_mode();
            state.set_status(&format!("API 모드: {}", state.api_mode.as_str()));
        }
        AppAction::ApiModeChanged(mode) => {
            state.api_mode = mode.clone();
            state.config.api_mode = mode;
            state.set_status("API 모드 변경");
        }
        AppAction::UpdateSearchFilter(term) => handle_search_filter(state, term),
        AppAction::SelectProject(project_id) => handle_select_project(state, project_id),
        AppAction::CreateProject(name) => handle_create_project(state, name),
        AppAction::ToggleClassificationScheme(scheme) => {
            state.set_status(&format!("분류 스킴 토글: {}", scheme));
        }
        AppAction::ExportRequested(format) => handle_export(state, format),
        AppAction::UpdateDocument(id, doc) => handle_update_document(state, id, doc),
        AppAction::DeleteDocument(id) => handle_delete_document(state, id),
        AppAction::SaveConfig => handle_save_config(state),
        AppAction::StartMetadataExtraction(path) => handle_drag(state, path),
        AppAction::SortBySimilarity(ref_id) => handle_sort_by_similarity(state, ref_id),
        AppAction::ClearSimilaritySort => handle_clear_similarity_sort(state),
        AppAction::OperationFailed(msg) => state.set_status(&format!("오류: {}", msg)),
        AppAction::Tick => {}

        AppAction::StartCitationExtraction { doc_id } => handle_start_citation_extraction(state, doc_id),
        AppAction::CitationExtracted { doc_id, edge_count, unmatched_count } => {
            state.finish_processing(&format!("인용 추출 완료: {}건 매치, {}건 미매치 (doc {})", edge_count, unmatched_count, doc_id));
        }
        AppAction::CitationExtractionFailed { doc_id, reason } => {
            state.finish_processing(&format!("인용 추출 실패 (doc {}): {}", doc_id, reason));
        }

        AppAction::StartManualCitationEntry { doc_id } => handle_start_manual_citation_entry(state, doc_id),
        AppAction::ManualCitationSaved { source_id, target_id } => {
            state.citation_entry_mode = false;
            state.set_status(&format!("인용 관계 저장: {} → {}", source_id, target_id));
            state.dirty = true;
        }
        AppAction::StartBibtexImport { doc_id, path } => handle_bibtex_import(state, doc_id, &path),
        AppAction::BibtexImported { doc_id, entry_count } => {
            state.bibtex_import_mode = false;
            state.bibtex_import_input.clear();
            state.set_status(&format!("BibTeX 가져오기 완료: {}건 (doc {})", entry_count, doc_id));
            state.dirty = true;
        }

        AppAction::GenerateCitationGraph { doc_ids } => handle_generate_citation_graph(state, doc_ids),
        AppAction::CitationGraphReady { graph_state: gs, cache_key: _, cache_hit } => {
            let suffix = if cache_hit { "(캐시)" } else { "(새로 생성)" };
            state.graph_state = Some(*gs);
            state.active_panel = PanelFocus::Graph;
            state.finish_processing(&format!("인용 그래프 준비 완료 {}", suffix));
        }
        AppAction::ToggleGraphRenderMode => {
            if let Some(ref mut gs) = state.graph_state {
                gs.cycle_render_mode();
                state.dirty = true;
            }
        }
        AppAction::NavigateGraph { direction } => handle_navigate_graph(state, direction),
        AppAction::SelectGraphNode { node_idx } => {
            if let Some(ref mut gs) = state.graph_state {
                gs.focused_node = Some(node_idx);
                state.dirty = true;
            }
        }
        AppAction::ExitGraphView => {
            state.graph_state = None;
            state.active_panel = PanelFocus::Right;
            state.set_status("준비됨");
            state.dirty = true;
        }
        AppAction::MouseHover { column, row } => {
            if !is_modal_active(state) {
                handle_mouse_hover(state, column, row);
            }
        }
        AppAction::MouseClick { column, row } => {
            if !is_modal_active(state) && state.graph_state.is_none() {
                handle_mouse_click(state, column, row);
            }
        }
        AppAction::TerminalResize { width, height } => {
            state.terminal_size = (width, height);
            state.dirty = true;
        }
    }
    false
}

fn handle_key(state: &mut AppState, key: KeyEvent) -> bool {
    if key.kind == KeyEventKind::Release {
        return false;
    }

    if state.show_help {
        state.show_help = false;
        state.dirty = true;
        return false;
    }

    if state.citation_entry_mode {
        return handle_citation_entry_key(state, key);
    }
    if state.bibtex_import_mode {
        return handle_bibtex_import_key(state, key);
    }
    if state.graph_state.is_some() {
        return handle_graph_key(state, key);
    }
    if state.search_mode {
        return handle_search_key(state, key);
    }
    if state.add_file_mode {
        return handle_add_file_key(state, key);
    }
    if state.new_project_mode {
        return handle_new_project_key(state, key);
    }
    if state.edit_mode {
        return handle_edit_key(state, key);
    }
    if state.note_mode {
        return handle_note_key(state, key);
    }
    if state.show_detail {
        return handle_detail_key(state, key);
    }

    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Esc => {
            if state.is_similarity_sorted() {
                let _ = state.action_tx.try_send(AppAction::ClearSimilaritySort);
            } else {
                return true;
            }
        }
        KeyCode::Tab => cycle_panel(state, true),
        KeyCode::Right => cycle_panel(state, true),
        KeyCode::Left => cycle_panel(state, false),
        KeyCode::Char('j') | KeyCode::Down => move_cursor_down(state),
        KeyCode::Char('k') | KeyCode::Up => move_cursor_up(state),
        KeyCode::Char(' ') => toggle_select(state),
        KeyCode::Char('/') => {
            state.search_mode = true;
            state.search_input.clear();
            state.dirty = true;
        }
        KeyCode::Char('?') => {
            state.show_help = true;
            state.dirty = true;
        }
        KeyCode::Char('a') => {
            state.add_file_mode = true;
            state.add_file_input.clear();
            state.set_status("파일 경로 입력 후 Enter");
            state.dirty = true;
        }
        KeyCode::Char('n') => {
            state.new_project_mode = true;
            state.new_project_input.clear();
            state.set_status("프로젝트 이름 입력 후 Enter");
            state.dirty = true;
        }
        KeyCode::Char('o') => {
            let _ = state.action_tx.try_send(AppAction::ToggleApiMode);
        }
        KeyCode::Char('x') => {
            if !state.selected_doc_ids.is_empty() {
                let _ = state.action_tx.try_send(AppAction::ExportRequested(ExportFormat::Bibtex));
            }
        }
        KeyCode::Char('s') => {
            if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor) {
                    if let Some(id) = doc.id {
                        let _ = state.action_tx.try_send(AppAction::SortBySimilarity(id));
                    }
                }
        }
        KeyCode::Char('d') => {
            if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor)
                    && let Some(id) = doc.id {
                        let _ = state.action_tx.try_send(AppAction::DeleteDocument(id));
                    }
        }
        KeyCode::Char('e') => {
            if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor).cloned()
                    && doc.id.is_some() {
                        state.edit_mode = true;
                        state.edit_field = 0;
                        state.edit_doc_id = doc.id;
                        state.edit_input = doc.title.clone();
                        state.set_status("편집: Tab으로 필드 이동, Enter로 저장");
                        state.dirty = true;
                    }
        }
        KeyCode::Char('g') => {
            if !state.selected_doc_ids.is_empty() {
                let doc_ids: Vec<i64> = state.selected_doc_ids.iter().copied().collect();
                let _ = state.action_tx.try_send(AppAction::GenerateCitationGraph { doc_ids });
            } else if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor)
                    && let Some(id) = doc.id {
                        let _ = state.action_tx.try_send(AppAction::GenerateCitationGraph { doc_ids: vec![id] });
                    }
        }
        KeyCode::Char('G') => {
            if let Some(ref gs) = state.graph_state {
                let doc_ids = gs.doc_ids.clone();
                let _ = state.action_tx.try_send(AppAction::GenerateCitationGraph { doc_ids });
            }
        }
        KeyCode::Char('C') => {
            if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor)
                    && let Some(id) = doc.id {
                        let _ = state.action_tx.try_send(AppAction::StartManualCitationEntry { doc_id: id });
                    }
        }
        KeyCode::Enter => {
            if state.active_panel == PanelFocus::Left {
                handle_tree_activate(state);
            } else if state.active_panel == PanelFocus::Right {
                state.show_detail = !state.show_detail;
                if state.show_detail {
                    state.load_detail();
                    state.active_panel = PanelFocus::Detail;
                } else {
                    state.detail_doc = None;
                }
                state.dirty = true;
            }
        }
        _ => {}
    }
    false
}

fn move_cursor_down(state: &mut AppState) {
    match state.active_panel {
        PanelFocus::Right => {
            if state.list_cursor + 1 < state.documents.len() {
                state.list_cursor += 1;
                if state.show_detail {
                    state.load_detail();
                }
                state.dirty = true;
            }
        }
        PanelFocus::Left => {
            let visible = count_tree_nodes(state);
            if state.tree_cursor + 1 < visible {
                state.tree_cursor += 1;
                state.dirty = true;
            }
        }
        PanelFocus::Detail | PanelFocus::Graph => {}
    }
}

fn move_cursor_up(state: &mut AppState) {
    match state.active_panel {
        PanelFocus::Right => {
            if state.list_cursor > 0 {
                state.list_cursor -= 1;
                if state.show_detail {
                    state.load_detail();
                }
                state.dirty = true;
            }
        }
        PanelFocus::Left => {
            if state.tree_cursor > 0 {
                state.tree_cursor -= 1;
                state.dirty = true;
            }
        }
        PanelFocus::Detail | PanelFocus::Graph => {}
    }
}

fn toggle_select(state: &mut AppState) {
    if state.active_panel == PanelFocus::Right
        && let Some(doc) = state.documents.get(state.list_cursor) {
            let id = doc.id.unwrap_or(0);
            if state.selected_doc_ids.contains(&id) {
                state.selected_doc_ids.remove(&id);
            } else {
                state.selected_doc_ids.insert(id);
            }
            state.dirty = true;
        }
}

fn cycle_panel(state: &mut AppState, forward: bool) {
    if state.show_detail {
        let pair = (PanelFocus::Right, PanelFocus::Detail);
        state.active_panel = cycle_pair(state.active_panel, pair, forward);
    } else if state.graph_state.is_some() {
        let pair = (PanelFocus::Right, PanelFocus::Graph);
        state.active_panel = cycle_pair(state.active_panel, pair, forward);
    } else {
        let pair = (PanelFocus::Left, PanelFocus::Right);
        state.active_panel = cycle_pair(state.active_panel, pair, forward);
    }
    state.dirty = true;
}

fn cycle_pair(current: PanelFocus, (a, b): (PanelFocus, PanelFocus), forward: bool) -> PanelFocus {
    if current == a {
        if forward { b } else { a }
    } else if current == b {
        if forward { b } else { a }
    } else {
        a
    }
}

fn handle_detail_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            state.show_detail = false;
            state.detail_doc = None;
            state.active_panel = PanelFocus::Right;
            state.dirty = true;
        }
        KeyCode::Enter => {
            if state.active_panel == PanelFocus::Detail {
                state.note_mode = true;
                state.note_input = state.current_note.clone().unwrap_or_default();
                state.dirty = true;
            } else {
                state.show_detail = false;
                state.detail_doc = None;
                state.active_panel = PanelFocus::Right;
                state.dirty = true;
            }
        }
        KeyCode::Char('n') => {
            state.note_mode = true;
            state.note_input = state.current_note.clone().unwrap_or_default();
            state.dirty = true;
        }
        KeyCode::Tab | KeyCode::Right => cycle_panel(state, true),
        KeyCode::Left => cycle_panel(state, false),
        KeyCode::Char('j') | KeyCode::Down => move_cursor_down(state),
        KeyCode::Char('k') | KeyCode::Up => move_cursor_up(state),
        _ => {}
    }
    false
}

fn handle_note_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            let doc_id = state.detail_doc.as_ref().and_then(|d| d.id).unwrap_or(0);
            if let Ok(conn) = state.db.lock() {
                let _ = crate::db::notes::set(&conn, doc_id, &state.note_input);
                state.current_note = Some(state.note_input.clone());
            }
            state.note_mode = false;
            state.set_status("노트 저장됨");
        }
        KeyCode::Enter => {
            state.note_input.push('\n');
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.note_input.push(c);
            state.dirty = true;
        }
        KeyCode::Backspace => {
            state.note_input.pop();
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_search_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.search_mode = false;
            state.search_input.clear();
            state.reload_documents();
        }
        KeyCode::Enter => {
            state.search_mode = false;
            if !state.search_input.is_empty() {
                let term = state.search_input.clone();
                let _ = state.action_tx.try_send(AppAction::UpdateSearchFilter(term));
            } else {
                state.reload_documents();
            }
        }
        KeyCode::Backspace => {
            state.search_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.search_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_add_file_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.add_file_mode = false;
            state.add_file_input.clear();
            state.set_status("준비됨");
        }
        KeyCode::Enter => {
            state.add_file_mode = false;
            let input = state.add_file_input.clone();
            state.add_file_input.clear();
            if input.is_empty() {
                state.set_status("준비됨");
            } else if let Some(path) = crate::terminal::drag_drop::parse_dragged_path(&input) {
                handle_drag(state, path);
            } else {
                state.set_status("파일을 찾을 수 없음");
            }
        }
        KeyCode::Backspace => {
            state.add_file_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.add_file_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_new_project_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.new_project_mode = false;
            state.new_project_input.clear();
            state.set_status("준비됨");
        }
        KeyCode::Enter => {
            state.new_project_mode = false;
            let name = state.new_project_input.clone();
            state.new_project_input.clear();
            if !name.is_empty() {
                let _ = state.action_tx.try_send(AppAction::CreateProject(name));
            } else {
                state.set_status("준비됨");
            }
        }
        KeyCode::Backspace => {
            state.new_project_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.new_project_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_edit_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.edit_mode = false;
            state.edit_doc_id = None;
            state.set_status("준비됨");
        }
        KeyCode::Enter => {
            let doc_id = state.edit_doc_id;
            state.edit_mode = false;
            state.edit_doc_id = None;

            if let Some(id) = doc_id {
                let updated = {
                    if let Ok(conn) = state.db.lock() {
                        if let Ok(Some(mut doc)) = documents::get_by_id(&conn, id) {
                            apply_edit_to_doc(&mut doc, state.edit_field, &state.edit_input);
                            Some(doc)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
                if let Some(doc) = updated {
                    let _ = state.action_tx.try_send(AppAction::UpdateDocument(id, Box::new(doc)));
                }
            }
            state.edit_input.clear();
        }
        KeyCode::Tab => {
            state.edit_field = (state.edit_field + 1) % EDIT_FIELDS.len();
            let doc_opt = state.documents.get(state.list_cursor).cloned();
            if let Some(doc) = doc_opt {
                state.edit_input = get_edit_field_value(&doc, state.edit_field);
            }
            state.dirty = true;
        }
        KeyCode::Backspace => {
            state.edit_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.edit_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn get_edit_field_value(doc: &documents::Document, field: usize) -> String {
    match field {
        0 => doc.title.clone(),
        1 => doc.authors.clone().unwrap_or_default(),
        2 => doc.journal.clone().unwrap_or_default(),
        3 => doc.conference.clone().unwrap_or_default(),
        4 => doc.pub_year.map(|y| y.to_string()).unwrap_or_default(),
        5 => doc.doi.clone().unwrap_or_default(),
        6 => doc.arxiv_id.clone().unwrap_or_default(),
        7 => doc.abstract_text.clone().unwrap_or_default(),
        8 => doc.keywords.clone().unwrap_or_default(),
        _ => String::new(),
    }
}

fn apply_edit_to_doc(doc: &mut documents::Document, field: usize, value: &str) {
    let trimmed = value.trim().to_string();
    match field {
        0 => doc.title = trimmed,
        1 => doc.authors = if trimmed.is_empty() { None } else { Some(trimmed) },
        2 => doc.journal = if trimmed.is_empty() { None } else { Some(trimmed) },
        3 => doc.conference = if trimmed.is_empty() { None } else { Some(trimmed) },
        4 => doc.pub_year = trimmed.parse::<i64>().ok(),
        5 => doc.doi = if trimmed.is_empty() { None } else { Some(trimmed) },
        6 => doc.arxiv_id = if trimmed.is_empty() { None } else { Some(trimmed) },
        7 => doc.abstract_text = if trimmed.is_empty() { None } else { Some(trimmed) },
        8 => doc.keywords = if trimmed.is_empty() { None } else { Some(trimmed) },
        _ => {}
    }
}

fn handle_tree_activate(state: &mut AppState) {
    let offset = 2 + state.projects.len().max(1);
    let tree_idx = state.tree_cursor.saturating_sub(offset);
    if tree_idx < UDC_TOP_LEVEL_STRS.len() {
        let notation = UDC_TOP_LEVEL_STRS[tree_idx].to_string();
        if state.expanded_nodes.contains(&notation) {
            state.expanded_nodes.remove(&notation);
        } else {
            state.expanded_nodes.insert(notation);
        }
        state.dirty = true;
    }
}

fn is_modal_active(state: &AppState) -> bool {
    state.edit_mode
        || state.search_mode
        || state.add_file_mode
        || state.new_project_mode
        || state.citation_entry_mode
        || state.bibtex_import_mode
        || state.show_help
}

fn compute_body_rect(state: &AppState) -> Rect {
    let (w, h) = state.terminal_size;
    let area = Rect { x: 0, y: 0, width: w, height: h };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .split(area);
    chunks[1]
}

fn compute_right_panel_rect(state: &AppState) -> Rect {
    let body = compute_body_rect(state);

    if state.show_detail {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Length(1), Constraint::Min(1)])
            .split(body);
        split[0]
    } else {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(32), Constraint::Length(1), Constraint::Min(1)])
            .split(body);
        split[2]
    }
}

fn compute_left_panel_rect(state: &AppState) -> Option<Rect> {
    if state.show_detail {
        return None;
    }
    let body = compute_body_rect(state);
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(32), Constraint::Length(1), Constraint::Min(1)])
        .split(body);
    Some(split[0])
}

fn compute_list_offset(cursor: usize, visible_items: usize, total_items: usize) -> usize {
    if total_items == 0 || visible_items == 0 {
        return 0;
    }
    let mut offset = 0usize;
    if cursor >= offset + visible_items {
        offset = cursor - visible_items + 1;
    }
    if offset + visible_items > total_items && total_items > visible_items {
        offset = total_items - visible_items;
    }
    offset
}

fn handle_mouse_hover(state: &mut AppState, column: u16, row: u16) {
    let right_rect = compute_right_panel_rect(state);
    let left_rect = compute_left_panel_rect(state);

    if is_in_rect(column, row, &right_rect) {
        let visible_items = (right_rect.height as usize) / 3;
        let total = state.documents.len();
        if total == 0 || visible_items == 0 {
            return;
        }
        let offset = compute_list_offset(state.list_cursor, visible_items, total);
        let rel_row = (row - right_rect.y) as usize;
        let item_index = offset + rel_row / 3;
        if item_index < total && item_index != state.list_cursor {
            state.list_cursor = item_index;
            if state.show_detail {
                state.load_detail();
            }
            state.dirty = true;
        }
    } else if let Some(left_rect) = left_rect
        && is_in_rect(column, row, &left_rect) {
        let visible_items = left_rect.height as usize;
        let total = count_tree_nodes(state);
        if total == 0 || visible_items == 0 {
            return;
        }
        let offset = compute_list_offset(state.tree_cursor, visible_items, total);
        let rel_row = (row - left_rect.y) as usize;
        let item_index = offset + rel_row;
        if item_index < total && item_index != state.tree_cursor {
            state.tree_cursor = item_index;
            state.dirty = true;
        }
    }
}

fn handle_mouse_click(state: &mut AppState, column: u16, row: u16) {
    let right_rect = compute_right_panel_rect(state);
    let left_rect = compute_left_panel_rect(state);

    if is_in_rect(column, row, &right_rect) {
        let visible_items = (right_rect.height as usize) / 3;
        let total = state.documents.len();
        if total == 0 || visible_items == 0 {
            return;
        }
        let offset = compute_list_offset(state.list_cursor, visible_items, total);
        let rel_row = (row - right_rect.y) as usize;
        let item_index = offset + rel_row / 3;
        if item_index < total {
            state.list_cursor = item_index;
            state.show_detail = !state.show_detail;
            if state.show_detail {
                state.load_detail();
                state.active_panel = PanelFocus::Detail;
            } else {
                state.detail_doc = None;
            }
            state.dirty = true;
        }
    } else if let Some(left_rect) = left_rect
        && is_in_rect(column, row, &left_rect) {
        let visible_items = left_rect.height as usize;
        let total = count_tree_nodes(state);
        if total == 0 || visible_items == 0 {
            return;
        }
        let offset = compute_list_offset(state.tree_cursor, visible_items, total);
        let rel_row = (row - left_rect.y) as usize;
        let item_index = offset + rel_row;
        if item_index < total {
            state.tree_cursor = item_index;
            state.active_panel = PanelFocus::Left;
            handle_tree_activate(state);
        }
    }
}

fn is_in_rect(column: u16, row: u16, rect: &Rect) -> bool {
    column >= rect.x
        && column < rect.x + rect.width
        && row >= rect.y
        && row < rect.y + rect.height
}

fn handle_drag(state: &mut AppState, path: PathBuf) {
    state.start_processing(&format!("처리 중: {}", path.display()));

    let tx = state.action_tx.clone();
    let path_clone = path.clone();

    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || pdf::process_file(&path_clone)).await;

        match result {
            Ok(Ok(meta)) => {
                let _ = tx.send(AppAction::MetadataExtracted(Box::new(meta), path)).await;
            }
            Ok(Err(e)) => {
                let _ = tx.send(AppAction::OperationFailed(e.to_string())).await;
            }
            Err(e) => {
                let _ = tx.send(AppAction::OperationFailed(format!("태스크 실패: {}", e))).await;
            }
        }
    });
}

enum MetaResult {
    Saved(i64),
    Duplicate(String),
    Failed(String),
    DbLockFailed,
}

fn handle_metadata(state: &mut AppState, meta: Box<pdf::RawMetadata>, original_path: PathBuf) {
    let library_path = state.config.library_path.clone();
    let mode = state.config.to_citation_key_mode();

    let result = {
        let conn_guard = state.db.lock();
        match conn_guard {
            Ok(conn) => {
                let is_dup = if let Some(ref doi) = meta.doi {
                    matches!(documents::find_by_doi(&conn, doi), Ok(Some(_)))
                } else {
                    false
                };
                if is_dup {
                    MetaResult::Duplicate("이미 등록된 문헌 (DOI 중복)".to_string())
                } else {
                    process_metadata_inner(&conn, &meta, &original_path, &library_path, &mode)
                }
            }
            Err(_) => MetaResult::DbLockFailed,
        }
    };

    match result {
        MetaResult::Saved(id) => {
            let _ = state.action_tx.try_send(AppAction::MetadataSaved(id));
        }
        MetaResult::Duplicate(msg) => {
            state.finish_processing(&msg);
        }
        MetaResult::Failed(msg) => {
            state.finish_processing(&format!("저장 실패: {}", msg));
        }
        MetaResult::DbLockFailed => {
            state.finish_processing("DB 락 획득 실패");
        }
    }
}

fn process_metadata_inner(
    conn: &rusqlite::Connection,
    meta: &pdf::RawMetadata,
    original_path: &std::path::Path,
    library_path: &std::path::Path,
    mode: &crate::citation::CitationKeyMode,
) -> MetaResult {
    let hash = library::compute_file_hash(original_path).ok();
    if let Some(ref h) = hash
        && let Ok(Some(_)) = documents::find_by_hash(conn, h) {
            return MetaResult::Duplicate("이미 등록된 파일 (해시 중복)".to_string());
        }

    let title = meta.title.clone().unwrap_or_else(|| "Untitled".to_string());
    let authors = if meta.authors.is_empty() {
        None
    } else {
        Some(meta.authors.join("; "))
    };

    let temp_doc = documents::Document {
        id: None,
        title,
        authors,
        journal: meta.journal.clone(),
        pub_year: meta.pub_year,
        doi: meta.doi.clone(),
        arxiv_id: meta.arxiv_id.clone(),
        abstract_text: meta.abstract_text.clone(),
        keywords: None,
        conference: None,
        file_path: None,
        file_hash: hash,
        citation_key: None,
        source: Some("pdf_extract".to_string()),
    };

    let key = generate_citation_key(&temp_doc, mode, |k| {
        documents::citation_key_exists(conn, k).unwrap_or(false)
    });

    let filename = library::build_library_filename(&key, "pdf");
    let file_path = library::copy_to_library(original_path, library_path, &filename)
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let doc = documents::Document {
        citation_key: Some(key),
        file_path,
        ..temp_doc
    };

    match documents::insert(conn, &doc) {
        Ok(id) => {
            let doc_for_class = documents::Document {
                id: Some(id),
                ..doc
            };
            if let Ok(recs) = crate::classification::recommender::recommend(conn, &doc_for_class, 3)
            {
                for rec in recs.iter().take(1) {
                    if let Ok(Some(node_id)) = crate::classification::scheme::get_node_id(
                        conn,
                        rec.scheme_code.as_str(),
                        &rec.notation,
                    ) {
                        let _ = crate::classification::scheme::assign_classification(
                            conn,
                            id,
                            node_id,
                            true,
                            Some(rec.confidence),
                            "auto",
                        );
                    }
                }
            }
            MetaResult::Saved(id)
        }
        Err(e) => MetaResult::Failed(e.to_string()),
    }
}

fn try_api_lookup(tx: tokio::sync::mpsc::Sender<AppAction>, mode: crate::api::ApiMode, doc: &documents::Document) {
    let doi = doc.doi.clone();
    let arxiv_id = doc.arxiv_id.clone();
    let doc_id = doc.id.unwrap_or(0);
    let title = doc.title.clone();

    tokio::spawn(async move {
        if let Some(doi) = doi {
            match crate::api::crossref::create_polite_http_client(None) {
                Ok(client) => {
                    match crate::api::crossref::fetch_by_doi(&client, &doi).await {
                        Ok(body) => {
                            if let Some(meta) = parse_crossref_response(&body) {
                                let _ = tx.send(AppAction::ApiLookupSuccess(meta, doc_id)).await;
                            } else {
                                let _ = tx.send(AppAction::ApiLookupSkipped("CrossRef 응답 파싱 실패".to_string())).await;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(AppAction::ApiLookupFailed(e.to_string())).await;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(AppAction::ApiLookupFailed(e.to_string())).await;
                }
            }
        } else if let Some(arxiv) = arxiv_id {
            match crate::api::arxiv::create_client() {
                Ok(client) => {
                    match crate::api::arxiv::fetch_by_arxiv_id(&client, &arxiv).await {
                        Ok(body) => {
                            if let Some(meta) = parse_arxiv_response(&body) {
                                let _ = tx.send(AppAction::ApiLookupSuccess(meta, doc_id)).await;
                            } else {
                                let _ = tx.send(AppAction::ApiLookupSkipped("arXiv 응답 파싱 실패".to_string())).await;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(AppAction::ApiLookupFailed(e.to_string())).await;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(AppAction::ApiLookupFailed(e.to_string())).await;
                }
            }
        } else if mode == crate::api::ApiMode::AutoFallback {
            match crate::api::crossref::create_polite_http_client(None) {
                Ok(client) => {
                    match crate::api::crossref::search_by_title(&client, &title).await {
                        Ok(body) => {
                            if let Some(meta) = parse_crossref_search_response(&body) {
                                let _ = tx.send(AppAction::ApiLookupSuccess(meta, doc_id)).await;
                            } else {
                                let _ = tx.send(AppAction::ApiLookupSkipped("제목 검색 결과 없음".to_string())).await;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(AppAction::ApiLookupFailed(e.to_string())).await;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(AppAction::ApiLookupFailed(e.to_string())).await;
                }
            }
        } else {
            let _ = tx.send(AppAction::ApiLookupSkipped("식별자 없음".to_string())).await;
        }
    });
}

fn parse_crossref_response(body: &str) -> Option<pdf::RawMetadata> {
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    let message = json.get("message")?;
    let title = message.get("title").and_then(|t| t.as_array()).and_then(|a| a.first()).and_then(|t| t.as_str());
    let authors = message.get("author").and_then(|a| a.as_array()).map(|arr| {
        arr.iter().filter_map(|a| {
            let family = a.get("family").and_then(|f| f.as_str()).unwrap_or("");
            let given = a.get("given").and_then(|g| g.as_str()).unwrap_or("");
            if !family.is_empty() {
                if given.is_empty() {
                    Some(family.to_string())
                } else {
                    Some(format!("{}, {}", family, given))
                }
            } else {
                a.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())
            }
        }).collect::<Vec<String>>()
    }).unwrap_or_default();

    let journal = message.get("container-title").and_then(|t| t.as_array()).and_then(|a| a.first()).and_then(|t| t.as_str());
    let year = message.get("published-print").or_else(|| message.get("published-online")).or_else(|| message.get("issued"))
        .and_then(|d| d.get("date-parts")).and_then(|d| d.as_array()).and_then(|a| a.first())
        .and_then(|a| a.as_array()).and_then(|a| a.first()).and_then(|y| y.as_i64());
    let doi = message.get("DOI").and_then(|d| d.as_str());
    let abstract_text = message.get("abstract").and_then(|a| a.as_str());

    Some(pdf::RawMetadata {
        title: title.map(|s| s.to_string()),
        authors,
        journal: journal.map(|s| s.to_string()),
        pub_year: year,
        doi: doi.map(|s| s.to_string()),
        arxiv_id: None,
        abstract_text: abstract_text.map(|s| s.to_string()),
        keywords: Vec::new(),
        source: pdf::MetadataSource::Crossref,
    })
}

fn parse_crossref_search_response(body: &str) -> Option<pdf::RawMetadata> {
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    let items = json.get("message").and_then(|m| m.get("items")).and_then(|i| i.as_array())?;
    let first = items.first()?;
    let title = first.get("title").and_then(|t| t.as_array()).and_then(|a| a.first()).and_then(|t| t.as_str());
    let authors = first.get("author").and_then(|a| a.as_array()).map(|arr| {
        arr.iter().filter_map(|a| {
            let family = a.get("family").and_then(|f| f.as_str()).unwrap_or("");
            let given = a.get("given").and_then(|g| g.as_str()).unwrap_or("");
            if !family.is_empty() {
                if given.is_empty() { Some(family.to_string()) } else { Some(format!("{}, {}", family, given)) }
            } else { None }
        }).collect::<Vec<String>>()
    }).unwrap_or_default();
    let journal = first.get("container-title").and_then(|t| t.as_array()).and_then(|a| a.first()).and_then(|t| t.as_str());
    let year = first.get("published-print").or_else(|| first.get("issued"))
        .and_then(|d| d.get("date-parts")).and_then(|d| d.as_array()).and_then(|a| a.first())
        .and_then(|a| a.as_array()).and_then(|a| a.first()).and_then(|y| y.as_i64());
    let doi = first.get("DOI").and_then(|d| d.as_str());

    Some(pdf::RawMetadata {
        title: title.map(|s| s.to_string()),
        authors,
        journal: journal.map(|s| s.to_string()),
        pub_year: year,
        doi: doi.map(|s| s.to_string()),
        arxiv_id: None,
        abstract_text: None,
        keywords: Vec::new(),
        source: pdf::MetadataSource::Crossref,
    })
}

fn parse_arxiv_response(body: &str) -> Option<pdf::RawMetadata> {
    let mut buf = Vec::new();

    let mut title: Option<String> = None;
    let mut authors: Vec<String> = Vec::new();
    let mut abstract_text: Option<String> = None;
    let mut year: Option<i64> = None;

    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(body);
    let mut in_title = false;
    let mut in_summary = false;
    let mut in_published = false;
    let mut in_name = false;
    let mut current_name = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "title" => in_title = true,
                    "summary" => in_summary = true,
                    "published" => in_published = true,
                    "name" => { in_name = true; current_name.clear(); }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "title" => in_title = false,
                    "summary" => in_summary = false,
                    "published" => in_published = false,
                    "name" => {
                        in_name = false;
                        if !current_name.is_empty() {
                            authors.push(current_name.clone());
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().ok().map(|s| s.to_string()).unwrap_or_default();
                if in_title && title.is_none() {
                    title = Some(text.clone());
                }
                if in_summary {
                    abstract_text = Some(text.clone());
                }
                if in_published
                    && let Some(y) = text.get(0..4).and_then(|s| s.parse::<i64>().ok()) {
                        year = Some(y);
                    }
                if in_name {
                    current_name = text;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    if title.is_none() && authors.is_empty() {
        return None;
    }

    Some(pdf::RawMetadata {
        title,
        authors,
        journal: None,
        pub_year: year,
        doi: None,
        arxiv_id: None,
        abstract_text,
        keywords: Vec::new(),
        source: pdf::MetadataSource::Arxiv,
    })
}

fn apply_api_metadata(state: &mut AppState, meta: pdf::RawMetadata, doc_id: i64) {
    let result = {
        if let Ok(conn) = state.db.lock() {
            if let Ok(Some(mut doc)) = documents::get_by_id(&conn, doc_id) {
                let mut changed = false;
                if (doc.title.is_empty() || doc.title == "Untitled")
                    && let Some(ref t) = meta.title {
                        doc.title = t.clone();
                        changed = true;
                    }
                if doc.authors.is_none() && !meta.authors.is_empty() {
                    doc.authors = Some(meta.authors.join("; "));
                    changed = true;
                }
                if doc.journal.is_none()
                    && let Some(ref j) = meta.journal {
                        doc.journal = Some(j.clone());
                        changed = true;
                    }
                if doc.pub_year.is_none()
                    && let Some(y) = meta.pub_year {
                        doc.pub_year = Some(y);
                        changed = true;
                    }
                if doc.doi.is_none()
                    && let Some(ref d) = meta.doi {
                        doc.doi = Some(d.clone());
                        changed = true;
                    }
                if doc.abstract_text.is_none()
                    && let Some(ref a) = meta.abstract_text {
                        doc.abstract_text = Some(a.clone());
                        changed = true;
                    }
                if changed {
                    match documents::update(&conn, &doc) {
                        Ok(()) => Some(Ok(())),
                        Err(e) => Some(Err(e.to_string())),
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    match result {
        Some(Ok(())) => {
            state.reload_documents();
            state.set_status("API 메타데이터 보강 완료");
        }
        Some(Err(msg)) => state.set_status(&format!("API 저장 실패: {}", msg)),
        None => state.set_status("API: 보강할 필드 없음"),
    }
}

fn handle_search_filter(state: &mut AppState, term: String) {
    state.search_input = term.clone();
    if term.is_empty() {
        state.reload_documents();
    } else {
        let docs = {
            if let Ok(conn) = state.db.lock() {
                if let Ok(ids) = crate::db::search::search_documents(&conn, &term) {
                    let mut docs = Vec::new();
                    for id in ids {
                        if let Ok(Some(doc)) = documents::get_by_id(&conn, id) {
                            docs.push(doc);
                        }
                    }
                    Some(docs)
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(docs) = docs {
            let count = docs.len();
            state.documents = docs;
            state.document_count = count;
            state.list_cursor = 0;
        }
    }
    state.dirty = true;
}

fn handle_select_project(state: &mut AppState, project_id: Option<i64>) {
    state.active_project_id = project_id;
    if let Some(pid) = project_id {
        let docs = {
            if let Ok(conn) = state.db.lock() {
                if let Ok(ids) = crate::db::projects::list_documents(&conn, pid) {
                    let mut docs = Vec::new();
                    for id in ids {
                        if let Ok(Some(doc)) = documents::get_by_id(&conn, id) {
                            docs.push(doc);
                        }
                    }
                    Some(docs)
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(docs) = docs {
            let count = docs.len();
            state.documents = docs;
            state.document_count = count;
            state.list_cursor = 0;
        }
    } else {
        state.reload_documents();
    }
    state.dirty = true;
}

fn handle_create_project(state: &mut AppState, name: String) {
    let result = {
        if let Ok(conn) = state.db.lock() {
            crate::db::projects::create_project(&conn, &name, None)
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(_id) => {
            state.reload_projects();
            state.set_status(&format!("프로젝트 생성: {}", name));
        }
        Err(e) => state.set_status(&format!("프로젝트 생성 실패: {}", e)),
    }
}

fn handle_export(state: &mut AppState, format: ExportFormat) {
    if state.selected_doc_ids.is_empty() {
        state.set_status("선택된 문헌이 없습니다");
        return;
    }

    let export_result = {
        if let Ok(conn) = state.db.lock() {
            let docs: Vec<documents::Document> = state
                .selected_doc_ids
                .iter()
                .filter_map(|id| documents::get_by_id(&conn, *id).ok().flatten())
                .collect();

            if docs.is_empty() {
                Some(Err("내보낼 문헌을 찾을 수 없습니다".to_string()))
            } else {
                let home = directories::BaseDirs::new()
                    .map(|d| d.home_dir().to_path_buf())
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                let filename = match format {
                    ExportFormat::Bibtex => "export.bib",
                    ExportFormat::CslJson => "export.json",
                };
                let export_path = home.join(filename);

                match std::fs::File::create(&export_path) {
                    Ok(mut file) => match export(&docs, format, &mut file) {
                        Ok(()) => Some(Ok(format!("내보내기 완료: {} ({}건)", export_path.display(), docs.len()))),
                        Err(e) => Some(Err(format!("내보내기 실패: {}", e))),
                    },
                    Err(e) => Some(Err(format!("파일 생성 실패: {}", e))),
                }
            }
        } else {
            Some(Err("DB 락 획득 실패".to_string()))
        }
    };

    if let Some(result) = export_result {
        match result {
            Ok(msg) => state.set_status(&msg),
            Err(msg) => state.set_status(&msg),
        }
    }
}

fn handle_update_document(state: &mut AppState, _id: i64, doc: Box<documents::Document>) {
    let result = {
        if let Ok(conn) = state.db.lock() {
            documents::update(&conn, &doc)
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(()) => {
            state.reload_documents();
            state.set_status("문헌 수정 완료");
        }
        Err(e) => state.set_status(&format!("수정 실패: {}", e)),
    }
}

fn handle_delete_document(state: &mut AppState, id: i64) {
    let result = {
        if let Ok(conn) = state.db.lock() {
            documents::delete(&conn, id)
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(()) => {
            state.selected_doc_ids.remove(&id);
            state.reload_documents();
            state.set_status("문헌 삭제 완료");
        }
        Err(e) => state.set_status(&format!("삭제 실패: {}", e)),
    }
}

fn handle_save_config(state: &mut AppState) {
    match state.config.save() {
        Ok(()) => state.set_status("설정 저장 완료 (~/.libran/config.toml)"),
        Err(e) => state.set_status(&format!("설정 저장 실패: {}", e)),
    }
}

fn handle_sort_by_similarity(state: &mut AppState, ref_id: i64) {
    state.start_processing("유사도 정렬 계산 중...");

    // Clone Arc to break borrow chain — MutexGuard borrows the clone, not state
    let db = state.db.clone();
    let conn = match db.lock() {
        Ok(c) => c,
        Err(_) => {
            state.finish_processing("DB 락 획득 실패");
            return;
        }
    };

    let all_docs = match documents::list_all(&conn) {
        Ok(docs) => docs,
        Err(e) => {
            // Lock is still alive here, but it borrows `db` (local), not `state`
            state.finish_processing(&format!("문헌 로드 실패: {e}"));
            return;
        }
    };

    let mut ref_title = String::new();
    let mut features = Vec::with_capacity(all_docs.len());

    for doc in &all_docs {
        let id = match doc.id {
            Some(id) => id,
            None => continue,
        };
        if id == ref_id {
            ref_title = doc.title.clone();
        }
        let udc_notations = get_udc_notations(&conn, id);
        let tags = documents::get_tags(&conn, id).unwrap_or_default();
        let cited_docs = documents::get_cited_docs(&conn, id).unwrap_or_default();
        let cited_by_docs = documents::get_citing_docs(&conn, id).unwrap_or_default();
        features.push(DocumentFeatures {
            id,
            udc_notations,
            tags,
            cited_docs,
            cited_by_docs,
            pub_year: doc.pub_year,
            conference: doc.conference.clone(),
        });
    }

    let doc_map: std::collections::HashMap<i64, crate::db::documents::Document> = all_docs
        .into_iter()
        .filter_map(|d| d.id.map(|id| (id, d)))
        .collect();

    // drop conn to release MutexGuard before computing scores
    drop(conn);

    let ref_features = match features.iter().find(|f| f.id == ref_id) {
        Some(f) => f.clone(),
        None => {
            state.finish_processing("기준 문헌을 찾을 수 없음");
            return;
        }
    };

    let scores = compute_scores(&ref_features, &features, &state.udc_tree, &state.similarity_config);

    let mut sorted_docs: Vec<crate::db::documents::Document> = scores
        .iter()
        .filter_map(|s| doc_map.get(&s.document_id).cloned())
        .collect();

    if let Some(ref_doc) = doc_map.get(&ref_id) {
        sorted_docs.insert(0, ref_doc.clone());
    }

    let short_title = ref_title.chars().take(40).collect::<String>();
    state.similarity_ref_doc_id = Some(ref_id);
    state.similarity_ref_title = ref_title;
    state.similarity_scores = scores;
    state.documents = sorted_docs;
    state.document_count = state.documents.len();
    state.list_cursor = 0;
    state.finish_processing(&format!("유사도 정렬 완료 (기준: {short_title})"));
}

fn handle_clear_similarity_sort(state: &mut AppState) {
    state.similarity_ref_doc_id = None;
    state.similarity_ref_title.clear();
    state.similarity_scores.clear();
    state.reload_documents();
    state.set_status("기본 정렬로 복귀");
}

/// Get UDC notation strings for a document from the classification DB.
fn get_udc_notations(conn: &rusqlite::Connection, document_id: i64) -> Vec<String> {
    let mut stmt = match conn.prepare(
        "SELECT cn.notation
         FROM document_classifications dc
         INNER JOIN classification_nodes cn ON dc.node_id = cn.id
         INNER JOIN classification_schemes cs ON cn.scheme_id = cs.id
         WHERE dc.document_id = ?1 AND cs.code = 'udc'",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = match stmt.query_map(rusqlite::params![document_id], |row| row.get::<_, String>(0)) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut notations = Vec::new();
    for row in rows.flatten() {
        // Parse compound UDC codes and add individual components
        notations.extend(crate::similarity::scoring::parse_udc_notation(&row));
    }
    notations
}

pub fn count_tree_nodes(state: &AppState) -> usize {
    let mut count = 2;
    count += state.projects.len().max(1);
    count += 1;
    count += 1;
    count += 9;
    for (notation, _) in UDC_TOP_LEVEL_TUPLES {
        if state.expanded_nodes.contains(*notation) {
            count += 3;
        }
    }

    let physh_has_docs = state.facets.iter().any(|f| f.scheme_code == "physh");

    if physh_has_docs {
        count += 1;
        count += 1;
        count += 4;
    }
    count
}

pub const UDC_TOP_LEVEL_STRS: &[&str] = &["0", "1", "2", "3", "5", "6", "7", "8", "9"];

pub const UDC_TOP_LEVEL_TUPLES: &[(&str, &str)] = &[
    ("0", "총류"),
    ("1", "철학"),
    ("2", "종교"),
    ("3", "사회과학"),
    ("5", "자연과학"),
    ("6", "응용과학"),
    ("7", "예술"),
    ("8", "언어"),
    ("9", "역사"),
];

pub const EDIT_FIELD_NAMES: &[&str] = EDIT_FIELDS;

fn handle_start_citation_extraction(state: &mut AppState, doc_id: i64) {
    state.start_processing(&format!("인용 추출 중 (doc {})...", doc_id));
    let db = state.db.clone();
    let tx = state.action_tx.clone();

    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || -> std::result::Result<(usize, usize), String> {
            let conn = db.lock().map_err(|e| e.to_string())?;

            if documents::has_reference_extraction(&conn, doc_id).unwrap_or(false) {
                return Ok((0, 0));
            }

            let file_path = documents::get_by_id(&conn, doc_id)
                .map_err(|e| e.to_string())?
                .and_then(|d| d.file_path)
                .ok_or_else(|| "파일 경로 없음".to_string())?;

            let text = crate::pdf::text::extract_text(std::path::Path::new(&file_path))
                .map_err(|e| e.to_string())?;

            let refs = extract::extract_references(&text);

            if refs.is_empty() {
                let _ = documents::save_reference_extraction(&conn, doc_id, &text, "heuristic_regex", 0);
                return Ok((0, 0));
            }

            let section_text: String = refs.iter().map(|r| r.raw_text.as_str()).collect::<Vec<_>>().join("\n");
            let _ = documents::save_reference_extraction(&conn, doc_id, &section_text, "heuristic_regex", 2);

            let fuzzy_threshold = 0.85;
            let mut edge_count = 0usize;
            let mut unmatched_count = 0usize;

            for r in &refs {
                match match_refs::match_reference_to_doc(&conn, r, fuzzy_threshold) {
                    Ok(Some(mr)) => {
                        let _ = entry::add_extracted_citation(
                            &conn,
                            doc_id,
                            mr.doc_id,
                            &mr.match_status,
                            mr.confidence,
                            Some(&r.raw_text),
                        );
                        edge_count += 1;
                    }
                    Ok(None) => {
                        unmatched_count += 1;
                    }
                    Err(_) => {
                        unmatched_count += 1;
                    }
                }
            }

            Ok((edge_count, unmatched_count))
        }).await;

        match result {
            Ok(Ok((edge_count, unmatched_count))) => {
                let _ = tx.send(AppAction::CitationExtracted { doc_id, edge_count, unmatched_count }).await;
            }
            Ok(Err(reason)) => {
                let _ = tx.send(AppAction::CitationExtractionFailed { doc_id, reason }).await;
            }
            Err(e) => {
                let _ = tx.send(AppAction::CitationExtractionFailed { doc_id, reason: e.to_string() }).await;
            }
        }
    });
}

fn handle_start_manual_citation_entry(state: &mut AppState, doc_id: i64) {
    state.citation_entry_mode = true;
    state.citation_entry_cursor = 0;
    state.edit_doc_id = Some(doc_id);
    state.set_status("인용 데이터 입력: Space로 선택, Enter로 저장");
    state.dirty = true;
}

fn handle_citation_entry_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.citation_entry_mode = false;
            state.edit_doc_id = None;
            state.set_status("준비됨");
            state.dirty = true;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if state.list_cursor + 1 < state.documents.len() {
                state.list_cursor += 1;
                state.dirty = true;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if state.list_cursor > 0 {
                state.list_cursor -= 1;
                state.dirty = true;
            }
        }
        KeyCode::Char(' ') => {
            let source_id = state.edit_doc_id.unwrap_or(0);
            if let Some(doc) = state.documents.get(state.list_cursor)
                && let Some(target_id) = doc.id
                    && source_id != target_id {
                        let db = state.db.clone();
                        if let Ok(conn) = db.lock() {
                            let _ = entry::add_manual_citation(&conn, source_id, target_id);
                            let _ = state.action_tx.try_send(AppAction::ManualCitationSaved {
                                source_id,
                                target_id,
                            });
                        }
                    }
        }
        KeyCode::Char('B') => {
            state.bibtex_import_mode = true;
            state.bibtex_import_input.clear();
            state.set_status("BibTeX 파일 경로 입력 후 Enter");
            state.dirty = true;
        }
        KeyCode::Enter => {
            state.citation_entry_mode = false;
            state.edit_doc_id = None;
            state.set_status("인용 데이터 입력 완료");
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_bibtex_import_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.bibtex_import_mode = false;
            state.bibtex_import_input.clear();
            state.set_status("준비됨");
            state.dirty = true;
        }
        KeyCode::Enter => {
            let path = state.bibtex_import_input.clone();
            let doc_id = state.edit_doc_id.unwrap_or(0);
            let _ = state.action_tx.try_send(AppAction::StartBibtexImport { doc_id, path });
        }
        KeyCode::Backspace => {
            state.bibtex_import_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.bibtex_import_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_bibtex_import(state: &mut AppState, doc_id: i64, path: &str) {
    state.start_processing("BibTeX 가져오기 중...");

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            state.finish_processing(&format!("BibTeX 파일 읽기 실패: {}", e));
            return;
        }
    };

    let bib_entries = entry::parse_bibtex(&content);

    let edge_count = if let Ok(conn) = state.db.lock() {
        let mut count = 0usize;
        for bib_entry in &bib_entries {
            if let Ok(Some(target_id)) = entry::match_bibtex_entry(&conn, bib_entry, 0.85) {
                let _ = entry::add_bibtex_citation(&conn, doc_id, target_id);
                count += 1;
            }
        }
        count
    } else {
        0
    };

    let _ = state.action_tx.try_send(AppAction::BibtexImported {
        doc_id,
        entry_count: edge_count,
    });
}

fn handle_generate_citation_graph(state: &mut AppState, doc_ids: Vec<i64>) {
    state.start_processing("인용 그래프 생성 중...");

    let db = state.db.clone();
    let tx = state.action_tx.clone();
    let doc_ids_clone = doc_ids.clone();

    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || -> std::result::Result<(CitationGraph, bool), String> {
            let conn = db.lock().map_err(|e| e.to_string())?;
            let cache_key = cache::build_cache_key(&doc_ids);

            let cache_hit = !cache::should_regenerate(&conn, &cache_key, &doc_ids)
                .unwrap_or(true);

            let graph = CitationGraph::build(&conn, &doc_ids)
                .map_err(|e| e.to_string())?;

            if !cache_hit {
                let edge_version = cache::compute_edge_version(&conn, &doc_ids).unwrap_or(0);
                let node_count = graph.node_count();
                let render_mode = graph::RenderMode::for_node_count(node_count);
                let graph_data = format!("{{\"nodes\":{},\"edges\":{}}}", node_count, graph.inner.edge_count());
                let _ = cache::store_cache(&conn, &cache_key, &graph_data, edge_version, doc_ids.len() as i64, &render_mode);
            }

            Ok((graph, cache_hit))
        }).await;

        match result {
            Ok(Ok((g, cache_hit))) => {
                let cache_key = cache::build_cache_key(&doc_ids_clone);
                let gs = GraphState::new(g, cache_hit);
                let _ = tx.send(AppAction::CitationGraphReady {
                    graph_state: Box::new(gs),
                    cache_key,
                    cache_hit,
                }).await;
            }
            Ok(Err(reason)) => {
                let _ = tx.send(AppAction::OperationFailed(format!("그래프 생성 실패: {}", reason))).await;
            }
            Err(e) => {
                let _ = tx.send(AppAction::OperationFailed(format!("태스크 실패: {}", e))).await;
            }
        }
    });
}

fn handle_graph_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            let _ = state.action_tx.try_send(AppAction::ExitGraphView);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let _ = state.action_tx.try_send(AppAction::NavigateGraph { direction: GraphDirection::Down });
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let _ = state.action_tx.try_send(AppAction::NavigateGraph { direction: GraphDirection::Up });
        }
        KeyCode::Char('h') | KeyCode::Left => {
            let _ = state.action_tx.try_send(AppAction::NavigateGraph { direction: GraphDirection::Left });
        }
        KeyCode::Char('l') | KeyCode::Right => {
            let _ = state.action_tx.try_send(AppAction::NavigateGraph { direction: GraphDirection::Right });
        }
        KeyCode::Tab => {
            let _ = state.action_tx.try_send(AppAction::ToggleGraphRenderMode);
        }
        KeyCode::Char('G') => {
            if let Some(ref gs) = state.graph_state {
                let doc_ids = gs.doc_ids.clone();
                let _ = state.action_tx.try_send(AppAction::GenerateCitationGraph { doc_ids });
            }
        }
        KeyCode::Enter => {
            if let Some(ref gs) = state.graph_state
                && let Some(node_idx) = gs.focused_node {
                    let _ = state.action_tx.try_send(AppAction::SelectGraphNode { node_idx });
                }
        }
        _ => {}
    }
    false
}

fn handle_navigate_graph(state: &mut AppState, direction: GraphDirection) {
    if let Some(ref mut gs) = state.graph_state {
        let step = match direction {
            GraphDirection::Down | GraphDirection::Right => 1,
            GraphDirection::Up | GraphDirection::Left => -1,
        };
        gs.focus_next(step);
        state.dirty = true;
    }
}
