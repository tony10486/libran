use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::citation::cache;
use crate::citation::entry;
use crate::citation::extract;
use crate::citation::generate_citation_key;
use crate::citation::graph;
use crate::citation::graph::CitationGraph;
use crate::citation::match_refs;
use crate::db::documents::{self, Document};
use crate::db::projects::Project;
use crate::export::{ExportFormat, export};
use crate::pdf;
use crate::similarity::scoring::{DocumentFeatures, compute_scores};
use crate::storage::library;
use crate::ui::help;
use tracing;

use super::AppState;
use super::action::{AppAction, GraphDirection};
use super::api_metadata::*;
use super::author_merge_handler;
use super::bookmark_handler;
use super::bulk_import_handler;
use super::custom_fields_handler::*;
use super::forward_citations_handler;
use super::graph_state::GraphState;
use super::import_handler;
use super::metrics_handler::*;
use super::reading_handler;
use super::saved_search_handler;
use super::state::PanelFocus;
use super::stats_handler;

pub const EDIT_FIELDS: &[&str] = &[
    "제목",
    "저자",
    "저널",
    "학회",
    "연도",
    "DOI",
    "arXiv",
    "초록",
    "키워드",
    "유형",
];

pub fn handle_action(state: &mut AppState, action: AppAction) -> bool {
    match action {
        AppAction::SystemShutdown => return true,
        AppAction::KeyPressed(key) => return handle_key(state, key),
        AppAction::DragDetected(path) => handle_drag(state, path),
        AppAction::MetadataExtracted(meta, original_path) => {
            handle_metadata(state, meta, original_path)
        }
        AppAction::MetadataSaved(id) => {
            let has_identifier = {
                if let Ok(conn) = state.db.lock() {
                    if let Ok(Some(doc)) = documents::get_by_id(&conn, id) {
                        doc.doi.is_some() || doc.arxiv_id.is_some()
                    } else {
                        false
                    }
                } else {
                    false
                }
            };
            if state.api_mode.allows_api_calls() && has_identifier {
                let doc_opt = {
                    if let Ok(conn) = state.db.lock() {
                        documents::get_by_id(&conn, id).ok().flatten()
                    } else {
                        None
                    }
                };
                if let Some(doc) = doc_opt {
                    try_api_lookup(state.action_tx.clone(), state.api_mode.clone(), &doc);
                    state.set_status("API 조회 중...");
                }
            } else {
                state.finish_processing("문헌 추가 완료");
            }
            state.reload_documents();
        }
        AppAction::ApiLookupSuccess(meta, doc_id) => apply_api_metadata(state, meta, doc_id),
        AppAction::ApiLookupFailed(msg) => state.finish_processing(&format!("API 실패: {}", msg)),
        AppAction::ApiLookupSkipped(msg) => state.finish_processing(&msg),
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

        AppAction::StartCitationExtraction { doc_id } => {
            handle_start_citation_extraction(state, doc_id)
        }
        AppAction::CitationExtracted {
            doc_id,
            edge_count,
            unmatched_count,
        } => {
            state.finish_processing(&format!(
                "인용 추출 완료: {}건 매치, {}건 미매치 (doc {})",
                edge_count, unmatched_count, doc_id
            ));
        }
        AppAction::CitationExtractionFailed { doc_id, reason } => {
            state.finish_processing(&format!("인용 추출 실패 (doc {}): {}", doc_id, reason));
        }

        AppAction::StartManualCitationEntry { doc_id } => {
            handle_start_manual_citation_entry(state, doc_id)
        }
        AppAction::ManualCitationSaved {
            source_id,
            target_id,
        } => {
            state.citation_entry_mode = false;
            state.set_status(&format!("인용 관계 저장: {} → {}", source_id, target_id));
            state.dirty = true;
        }
        AppAction::StartBibtexImport { doc_id, path } => handle_bibtex_import(state, doc_id, &path),
        AppAction::BibtexImported {
            doc_id,
            entry_count,
        } => {
            state.bibtex_import_mode = false;
            state.bibtex_import_input.clear();
            state.set_status(&format!(
                "BibTeX 가져오기 완료: {}건 (doc {})",
                entry_count, doc_id
            ));
            state.dirty = true;
        }

        AppAction::GenerateCitationGraph { doc_ids } => {
            handle_generate_citation_graph(state, doc_ids)
        }
        AppAction::CitationGraphReady {
            graph_state: gs,
            cache_key: _,
            cache_hit,
        } => {
            let suffix = if cache_hit {
                "(캐시)"
            } else {
                "(새로 생성)"
            };
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
            if state.settings_panel_mode {
                if crate::ui::settings_panel::handle_mouse_hover(state, column, row) {
                    state.dirty = true;
                }
            } else if !is_modal_active(state) {
                handle_mouse_hover(state, column, row);
            }
        }
        AppAction::MouseClick { column, row } => {
            if state.settings_panel_mode {
                if crate::ui::settings_panel::handle_mouse_click(state, column, row) {
                    state.dirty = true;
                }
            } else if !is_modal_active(state) && state.graph_state.is_none() {
                handle_mouse_click(state, column, row);
            }
        }
        AppAction::TerminalResize { width, height } => {
            state.terminal_size = (width, height);
            state.dirty = true;
        }
        AppAction::AddTag { doc_id, tag } => {
            let err = {
                state.db.lock().ok().and_then(|conn| {
                    documents::add_tag(&conn, doc_id, &tag).err()
                })
            };
            if let Some(e) = err {
                tracing::error!("add_tag 실패 (doc_id={}, tag={}): {}", doc_id, tag, e);
                state.set_status(&format!("태그 추가 실패: {}", e));
            } else {
                state.set_status(&format!("태그 추가: {}", tag));
            }
            state.reload_tags();
            state.dirty = true;
        }
        AppAction::RemoveTag { doc_id, tag } => {
            let err = {
                state.db.lock().ok().and_then(|conn| {
                    documents::remove_tag(&conn, doc_id, &tag).err()
                })
            };
            if let Some(e) = err {
                tracing::error!("remove_tag 실패 (doc_id={}, tag={}): {}", doc_id, tag, e);
                state.set_status(&format!("태그 삭제 실패: {}", e));
            } else {
                state.set_status(&format!("태그 삭제: {}", tag));
            }
            state.reload_tags();
            state.dirty = true;
        }
        AppAction::SetRating { doc_id, rating } => {
            let err = {
                state.db.lock().ok().and_then(|conn| {
                    documents::update_rating(&conn, doc_id, rating.map(|r| r as i64)).err()
                })
            };
            if let Some(e) = err {
                tracing::error!("update_rating 실패 (doc_id={}): {}", doc_id, e);
                state.set_status(&format!("평점 설정 실패: {}", e));
            }
            if let Some(doc) = state.documents.iter_mut().find(|d| d.id == Some(doc_id)) {
                doc.rating = rating.map(|r| r as i64);
            }
            if let Some(doc) = state.detail_doc.as_mut() {
                if doc.id == Some(doc_id) {
                    doc.rating = rating.map(|r| r as i64);
                }
            }
            let status = match rating {
                Some(r) => format!("별점 설정: {}점", r),
                None => "별점 삭제됨".to_string(),
            };
            state.set_status(&status);
            state.dirty = true;
        }
        AppAction::CreateSeries(name) => handle_create_series(state, name),
        AppAction::SelectSeries(series_id) => handle_select_series(state, series_id),
        AppAction::DeleteSeries(series_id) => handle_delete_series(state, series_id),
        AppAction::ToggleSeriesGrouping => handle_toggle_series_grouping(state),
        AppAction::AssignDocToSeries {
            doc_id,
            series_id,
            volume,
            issue,
        } => handle_assign_doc_to_series(state, doc_id, series_id, volume, issue),
        AppAction::AutoGroupSeries => handle_auto_group_series(state),
        AppAction::AddDocsToProject {
            project_id,
            doc_ids,
        } => handle_add_docs_to_project(state, project_id, doc_ids),
        AppAction::RemoveDocsFromProject {
            project_id,
            doc_ids,
        } => handle_remove_docs_from_project(state, project_id, doc_ids),
        AppAction::DeleteProject(project_id) => handle_delete_project(state, project_id),
        AppAction::SelectAuthor(author) => handle_select_author(state, author),
        AppAction::FetchAuthorMetrics { name } => handle_fetch_author_metrics(state, name),
        AppAction::AuthorMetricsFetched { name, metrics } => {
            handle_author_metrics_fetched(state, name, *metrics)
        }
        AppAction::AuthorMetricsFailed { name, reason } => {
            state.set_status(&format!("지표 조회 실패 ({}): {}", name, reason));
        }
        AppAction::SetMetricsBackend(backend) => handle_set_metrics_backend(state, backend),
        AppAction::RegisterApiKey(key) => handle_register_api_key(state, key),
        AppAction::ShowMetricsOverlay { name } => {
            state.show_metrics_overlay = true;
            state.metrics_overlay_name = name;
            state.dirty = true;
        }
        AppAction::CloseMetricsOverlay => {
            state.show_metrics_overlay = false;
            state.metrics_overlay_name.clear();
            state.dirty = true;
        }
        AppAction::LookupByDoi { doc_id } => {
            if state.api_mode.allows_api_calls() {
                let doc_opt = {
                    if let Ok(conn) = state.db.lock() {
                        documents::get_by_id(&conn, doc_id).ok().flatten()
                    } else {
                        None
                    }
                };
                if let Some(doc) = doc_opt {
                    state.start_processing("CrossRef로 메타데이터 조회 중...");
                    try_api_lookup(state.action_tx.clone(), state.api_mode.clone(), &doc);
                }
            } else {
                state.set_status("API 모드가 오프라인입니다 (o 키로 전환)");
            }
        }
        AppAction::SelectUdc(notation) => handle_select_udc(state, notation),
        AppAction::AddCustomField { doc_id, key, value } => {
            handle_add_custom_field(state, doc_id, key, value)
        }
        AppAction::DeleteCustomField { doc_id, field_id } => {
            handle_delete_custom_field(state, doc_id, field_id)
        }
        AppAction::OpenExternalViewer { doc_id } => handle_open_external(state, doc_id),
        AppAction::OpenExternalViewerResult {
            success: _,
            message,
        } => handle_open_external_result(state, &message),
        AppAction::InstallSioyek { doc_id } => handle_install_sioyek(state, doc_id),
        AppAction::SioyekInstallResult { success, message, doc_id } => {
            handle_sioyek_install_result(state, success, &message, doc_id);
        }
        AppAction::InstallOkular { doc_id } => handle_install_okular(state, doc_id),
        AppAction::OkularInstallResult { success, message, doc_id } => {
            handle_okular_install_result(state, success, &message, doc_id);
        }
        AppAction::ToggleReadingStatus { doc_id } => {
            reading_handler::handle_toggle_reading_status(state, doc_id)
        }
        AppAction::SaveCurrentSearch => saved_search_handler::handle_save_current_search(state),
        AppAction::SaveCurrentSearchNamed { name } => {
            saved_search_handler::handle_save_current_search_named(state, name)
        }
        AppAction::SelectSavedSearch { search_id } => {
            saved_search_handler::handle_select_saved_search(state, search_id)
        }
        AppAction::DeleteSavedSearch { search_id } => {
            saved_search_handler::handle_delete_saved_search(state, search_id)
        }
        AppAction::ToggleStatsDashboard => stats_handler::handle_toggle_stats_dashboard(state),
        AppAction::ExtractBookmarks { doc_id } => {
            bookmark_handler::handle_extract_bookmarks(state, doc_id)
        }
        AppAction::BookmarksExtracted { doc_id, bookmarks } => {
            bookmark_handler::handle_bookmarks_extracted(state, doc_id, bookmarks)
        }
        AppAction::BookmarkExtractionFailed { doc_id, reason } => {
            bookmark_handler::handle_bookmark_extraction_failed(state, doc_id, reason)
        }
        AppAction::StartBulkImport => bulk_import_handler::handle_start_bulk_import(state),
        AppAction::BulkImportSubmitted(input) => {
            bulk_import_handler::handle_bulk_import_submitted(state, input)
        }
        AppAction::BulkImportResult {
            success_count,
            fail_count,
            message,
        } => bulk_import_handler::handle_bulk_import_result(
            state,
            success_count,
            fail_count,
            message,
        ),
        AppAction::StartFileImport => import_handler::handle_start_file_import(state),
        AppAction::FileImportSubmitted(path) => {
            import_handler::handle_file_import_submitted(state, path)
        }
        AppAction::FileImportResult { count, message } => {
            import_handler::handle_file_import_result(state, count, message)
        }
        AppAction::FetchForwardCitations { doc_id } => {
            forward_citations_handler::handle_fetch_forward_citations(state, doc_id)
        }
        AppAction::ForwardCitationsFetched { doc_id, count } => {
            forward_citations_handler::handle_forward_citations_fetched(state, doc_id, count)
        }
        AppAction::ForwardCitationsFailed { doc_id, reason } => {
            forward_citations_handler::handle_forward_citations_failed(state, doc_id, reason)
        }
        AppAction::StartAuthorMerge => author_merge_handler::handle_start_author_merge(state),
        AppAction::AuthorMergeSourceEntered(source) => {
            author_merge_handler::handle_author_merge_source_entered(state, source)
        }
        AppAction::AuthorMergeCanonicalEntered { source, canonical } => {
            author_merge_handler::handle_author_merge_canonical_entered(state, source, canonical)
        }
        AppAction::AuthorMergeResult { success, message } => {
            author_merge_handler::handle_author_merge_result(state, success, message)
        }
        AppAction::Backup { path } => handle_backup(state, path),
        AppAction::Restore { path } => handle_restore(state, path),
        AppAction::ImportDb { path } => handle_import_db(state, path),
        AppAction::ProcessFinished { success, message } => {
            state.finish_processing(&message);
            if success {
                state.reload_documents();
            }
        }
        AppAction::ToggleFavoriteFilter => handle_toggle_favorite_filter(state),
        AppAction::SetTagColor { tag, color } => handle_set_tag_color(state, &tag, color),
        AppAction::ToggleQueueView => handle_toggle_queue_view(state),
        AppAction::AddToQueue { doc_id } => handle_add_to_queue(state, doc_id),
        AppAction::RemoveFromQueue { doc_id } => handle_remove_from_queue(state, doc_id),
        AppAction::ReorderQueue {
            doc_id,
            new_position,
        } => handle_reorder_queue(state, doc_id, new_position),
        AppAction::UpdateReadingProgress { doc_id, progress } => {
            handle_update_reading_progress(state, doc_id, progress)
        }
        AppAction::ImportClassification { path } => handle_import_classification(state, &path),

        // ── Widget Actions ──
        AppAction::ToggleWidgetPanel => {
            state.show_widget_panel = !state.show_widget_panel;
            state.dirty = true;
        }
        AppAction::WidgetTabNext => {
            state.widget_registry.next_tab();
            state.dirty = true;
        }
        AppAction::WidgetTabPrev => {
            state.widget_registry.prev_tab();
            state.dirty = true;
        }
        AppAction::WidgetTabSwitch(idx) => {
            state.widget_registry.switch_tab(idx);
            state.dirty = true;
        }
        AppAction::WidgetRefresh { widget_id } => {
            spawn_widget_fetch(state, &widget_id);
            state.dirty = true;
        }
        AppAction::WidgetDataReady { widget_id, content } => {
            state.widget_registry.set_content(&widget_id, *content);
            // API/Script runner의 last_refresh 갱신
            if let Some(instance) =
                state.widget_registry.widgets.iter_mut().find(|w| w.id() == widget_id)
            {
                match instance {
                    crate::widget::WidgetInstance::Api(runner) => runner.mark_refreshed(),
                    crate::widget::WidgetInstance::Script(runner) => runner.mark_refreshed(),
                }
            }
            state.dirty = true;
        }
        AppAction::WidgetError { widget_id, message } => {
            state.widget_registry.set_content(
                &widget_id,
                crate::widget::WidgetContent::error(message),
            );
            state.dirty = true;
        }
        AppAction::WidgetTick => {
            handle_widget_tick(state);
        }
    }
    false
}

pub(crate) fn normalize_korean_key(key: KeyEvent) -> KeyEvent {
    if let KeyCode::Char(c) = key.code {
        if let Some(mapped) = korean_to_qwerty(c) {
            return KeyEvent {
                code: KeyCode::Char(mapped),
                modifiers: key.modifiers,
                kind: key.kind,
                state: key.state,
            };
        }
    }
    key
}

fn korean_to_qwerty(c: char) -> Option<char> {
    const MAP: &[(char, char)] = &[
        ('ㅂ', 'q'),
        ('ㅈ', 'w'),
        ('ㄷ', 'e'),
        ('ㄱ', 'r'),
        ('ㅅ', 't'),
        ('ㅛ', 'y'),
        ('ㅕ', 'u'),
        ('ㅑ', 'i'),
        ('ㅐ', 'o'),
        ('ㅔ', 'p'),
        ('ㅁ', 'a'),
        ('ㄴ', 's'),
        ('ㅇ', 'd'),
        ('ㄹ', 'f'),
        ('ㅎ', 'g'),
        ('ㅗ', 'h'),
        ('ㅓ', 'j'),
        ('ㅏ', 'k'),
        ('ㅣ', 'l'),
        ('ㅋ', 'z'),
        ('ㅌ', 'x'),
        ('ㅊ', 'c'),
        ('ㅍ', 'v'),
        ('ㅠ', 'b'),
        ('ㅜ', 'n'),
        ('ㅡ', 'm'),
        ('ㅃ', 'Q'),
        ('ㅉ', 'W'),
        ('ㄸ', 'E'),
        ('ㄲ', 'R'),
        ('ㅆ', 'T'),
        ('ㅒ', 'O'),
        ('ㅖ', 'P'),
    ];
    MAP.iter().find(|(k, _)| *k == c).map(|(_, v)| *v)
}

fn handle_key(state: &mut AppState, key: KeyEvent) -> bool {
    if key.kind == KeyEventKind::Release {
        return false;
    }

    // Input modes: pass raw key (Korean text should be entered as-is)
    if state.show_help {
        let key = normalize_korean_key(key);
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                state.show_help = false;
                state.help_page = 0;
                state.dirty = true;
            }
            KeyCode::Tab
            | KeyCode::Right
            | KeyCode::Char('n')
            | KeyCode::Char('l')
            | KeyCode::PageDown
            | KeyCode::Char(' ') => {
                state.help_page = (state.help_page + 1) % help::TOTAL_PAGES;
                state.dirty = true;
            }
            KeyCode::BackTab
            | KeyCode::Left
            | KeyCode::Char('p')
            | KeyCode::Char('h')
            | KeyCode::PageUp => {
                state.help_page = (state.help_page + help::TOTAL_PAGES - 1) % help::TOTAL_PAGES;
                state.dirty = true;
            }
            _ => {}
        }
        return false;
    }

    if state.citation_entry_mode {
        return handle_citation_entry_key(state, key);
    }
    if state.bibtex_import_mode {
        return handle_bibtex_import_key(state, key);
    }
    if state.graph_state.is_some() {
        let key = normalize_korean_key(key);
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
    if state.new_series_mode {
        return handle_new_series_key(state, key);
    }
    if state.pick_project_mode {
        return handle_pick_project_key(state, key);
    }
    if state.author_search_mode {
        return handle_author_search_key(state, key);
    }
    if state.confirm_delete_mode {
        return handle_confirm_delete_key(state, key);
    }
    if state.show_sioyek_install_dialog {
        return handle_sioyek_install_key(state, key);
    }
    if state.show_okular_install_dialog {
        return handle_okular_install_key(state, key);
    }
    if state.api_key_input_mode {
        return handle_api_key_input_key(state, key);
    }
    if state.show_metrics_overlay {
        return handle_metrics_overlay_key(state, key);
    }
    if state.custom_field_mode {
        return handle_custom_field_key(state, key);
    }
    if state.show_export_dialog {
        let key = normalize_korean_key(key);
        return handle_export_dialog_key(state, key);
    }
    if state.edit_mode {
        return handle_edit_key(state, key);
    }
    if state.note_mode {
        return handle_note_key(state, key);
    }
    if state.tag_mode {
        return handle_tag_key(state, key);
    }
    if state.rating_mode {
        return handle_rating_key(state, key);
    }
    if state.save_search_mode {
        return handle_save_search_key(state, key);
    }
    if state.bulk_import_mode {
        return handle_bulk_import_key(state, key);
    }
    if state.file_import_mode {
        return handle_file_import_key(state, key);
    }
    if state.author_merge_mode {
        return handle_author_merge_key(state, key);
    }
    if state.show_stats {
        let key = normalize_korean_key(key);
        return handle_stats_overlay_key(state, key);
    }
    if state.queue_view {
        let key = normalize_korean_key(key);
        return handle_queue_view_key(state, key);
    }
    if state.settings_panel_mode {
        let key = normalize_korean_key(key);
        return crate::ui::settings_panel::handle_key(state, key);
    }
    if state.show_widget_panel {
        let key = normalize_korean_key(key);
        return handle_widget_panel_key(state, key);
    }
    if state.command_mode {
        return handle_command_key(state, key);
    }
    if state.show_detail {
        let key = normalize_korean_key(key);
        return handle_detail_key(state, key);
    }

    // Non-input modes: normalize Korean keys to QWERTY
    let key = normalize_korean_key(key);

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
            state.help_page = 0;
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
        KeyCode::Char('m') => {
            if state.selected_doc_ids.is_empty() {
                state.set_status("먼저 문헌을 선택하세요 (Space)");
            } else if state.projects.is_empty() {
                state.set_status("프로젝트가 없습니다 (n 키로 생성)");
            } else {
                state.pick_project_mode = true;
                state.pick_project_input.clear();
                state.pick_project_cursor = 0;
                state.set_status("프로젝트를 선택 후 Enter");
                state.dirty = true;
            }
        }
        KeyCode::Char('S') => {
            state.new_series_mode = true;
            state.new_series_input.clear();
            state.set_status("시리즈 이름 입력 후 Enter");
            state.dirty = true;
        }
        KeyCode::Char('M') => {
            let _ = state.action_tx.try_send(AppAction::ToggleSeriesGrouping);
        }
        KeyCode::Char('A') => {
            let _ = state.action_tx.try_send(AppAction::AutoGroupSeries);
        }
        KeyCode::Char('f') => {
            if state.authors_expanded && !state.authors.is_empty() {
                state.author_search_mode = true;
                state.author_search_input.clear();
                state.set_status("연구자 이름 검색");
                state.dirty = true;
            } else {
                state.set_status("먼저 연구자 섹션을 펼치세요 (Enter)");
            }
        }
        KeyCode::Char('H') => {
            let author_name = state.active_author.clone().or_else(|| {
                if state.active_panel == PanelFocus::Left && state.authors_expanded {
                    let filtered = filtered_authors(state);
                    let authors_start = count_authors_section_start(state);
                    let header_offset = if state.author_search_mode { 2 } else { 1 };
                    let list_start = authors_start + header_offset;
                    if state.tree_cursor >= list_start {
                        let idx = state.tree_cursor - list_start;
                        filtered.get(idx).map(|(n, _)| n.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
            if let Some(name) = author_name {
                if state.metrics_backend.requires_api_key()
                    && state.openalex_api_key.as_deref().unwrap_or("").is_empty()
                {
                    state.set_status("OpenAlex API 키가 필요합니다 (K 키로 등록)");
                } else {
                    state.set_status(&format!("{} 지표 조회 중...", name));
                    let _ = state
                        .action_tx
                        .try_send(AppAction::FetchAuthorMetrics { name });
                }
            } else {
                state.set_status("연구자를 먼저 선택하세요 (왼쪽 패널 Enter)");
            }
        }
        KeyCode::Char('K') => {
            state.api_key_input_mode = true;
            state.api_key_input.clear();
            state.set_status("OpenAlex API 키 입력 후 Enter (비워두면 백엔드 전환만)");
            state.dirty = true;
        }
        KeyCode::Char('B') => {
            state.auto_fetch_metrics = !state.auto_fetch_metrics;
            let enabled = state.auto_fetch_metrics;
            if let Ok(conn) = state.db.lock() {
                let val = if enabled { "true" } else { "false" };
                if let Err(e) = conn.execute(
                    "INSERT OR REPLACE INTO app_config (key, value, updated_at) \
                     VALUES ('auto_fetch_metrics', ?1, datetime('now'))",
                    rusqlite::params![val],
                ) {
                    tracing::error!("Failed to save auto_fetch_metrics setting: {e}");
                }
            }
            state.set_status(if enabled {
                "자동 지표 조회 켜짐 (저자 선택 시 자동 조회, 갱신 주기 7일)"
            } else {
                "자동 지표 조회 꺼짐"
            });
            state.dirty = true;
        }
        KeyCode::Char('O') => {
            let _ = state.action_tx.try_send(AppAction::ToggleApiMode);
        }
        KeyCode::Char('+') => {
            state.resize_left_panel(2);
        }
        KeyCode::Char('-') => {
            state.resize_left_panel(-2);
        }
        KeyCode::Char('=') => {
            state.reset_left_panel();
        }
        KeyCode::Char('p') => {
            state.settings_panel_mode = true;
            state.settings_panel = Some(crate::ui::settings_panel::SettingsPanelState {
                active_tab: 0,
                cursor: 0,
                editing: false,
                edit_input: String::new(),
                theme_dropdown_open: false,
                available_themes: Vec::new(),
                theme_cursor: 0,
                color_picker_open: false,
                picker_grid_x: 0,
                picker_grid_y: 0,
                picker_hue: 0.0,
                theme_color_cursor: 0,
            });
            state.dirty = true;
        }
        KeyCode::Char('w') => {
            state.show_widget_panel = !state.show_widget_panel;
            state.dirty = true;
        }
        KeyCode::Char(':') => {
            state.command_mode = true;
            state.command_input.clear();
            state.set_status(
                "명령 입력 (:backup / :restore / :import-db / :tag-color / :import-classification / :note)",
            );
            state.dirty = true;
        }
        KeyCode::Char('x') => {
            state.show_export_dialog = true;
            state.export_dialog_state.focused_section =
                crate::export::export_dialog_state::DialogSection::Scope;

            if !state.selected_doc_ids.is_empty() {
                state.export_dialog_state.selected_scope = crate::export::export_dialog_state::ExportScope::SelectedOnly;
            } else {
                state.export_dialog_state.selected_scope = crate::export::export_dialog_state::ExportScope::EntireLibrary;
            }

            if let Ok(conn) = state.db.lock() {
                let target_id = if let Some(&id) = state.selected_doc_ids.iter().next() {
                    Some(id)
                } else if let Some(doc) = state.documents.get(state.list_cursor) {
                    doc.id
                } else {
                    None
                };

                if let Some(id) = target_id {
                    if let Ok(Some(doc)) = documents::get_by_id(&conn, id) {
                        state.export_dialog_state.update_preview(&doc);
                    }
                }
            }
            state.dirty = true;
        }
        KeyCode::Char('s') => {
            if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor)
            {
                if let Some(id) = doc.id {
                    let _ = state.action_tx.try_send(AppAction::SortBySimilarity(id));
                }
            }
        }
        KeyCode::Char('d') => {
            if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor).cloned()
                && let Some(id) = doc.id
            {
                if state.skip_delete_confirm {
                    let _ = state.action_tx.try_send(AppAction::DeleteDocument(id));
                } else {
                    state.confirm_delete_mode = true;
                    state.delete_confirm_doc_id = Some(id);
                    state.delete_confirm_title = doc.title.clone();
                    state.dirty = true;
                }
            }
        }
        KeyCode::Char('D') => {
            if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor).cloned()
                && let Some(id) = doc.id
            {
                if !state.api_mode.allows_api_calls() {
                    state.set_status("API 모드가 오프라인입니다 (o 키로 전환)");
                } else if doc.doi.is_none() && doc.arxiv_id.is_none() {
                    state.set_status("DOI 또는 arXiv ID가 없습니다");
                } else {
                    state.start_processing("CrossRef로 메타데이터 조회 중...");
                    let _ = state
                        .action_tx
                        .try_send(AppAction::LookupByDoi { doc_id: id });
                }
            }
        }
        KeyCode::Char('e') => {
            if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor).cloned()
                && doc.id.is_some()
            {
                state.edit_mode = true;
                state.edit_field = 0;
                state.edit_doc_id = doc.id;
                state.edit_input = doc.title.clone();
                state.set_status("편집: Tab으로 필드 이동, Enter로 저장");
                state.dirty = true;
            }
        }
        KeyCode::Char('t') => {
            let doc_id = if state.active_panel == PanelFocus::Detail {
                state.detail_doc.as_ref().and_then(|d| d.id)
            } else if state.active_panel == PanelFocus::Right {
                state.documents.get(state.list_cursor).and_then(|d| d.id)
            } else {
                None
            };
            if doc_id.is_some() {
                state.load_detail();
                state.show_detail = true;
                state.active_panel = PanelFocus::Detail;
                state.tag_mode = true;
                state.tag_input = state.current_tags.join(" ");
                state.set_status("태그 편집 (스페이스 구분, Esc 저장)");
                state.dirty = true;
            }
        }
        KeyCode::Char('g') => {
            if !state.selected_doc_ids.is_empty() {
                let doc_ids: Vec<i64> = state.selected_doc_ids.iter().copied().collect();
                let _ = state
                    .action_tx
                    .try_send(AppAction::GenerateCitationGraph { doc_ids });
            } else if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor)
                && let Some(id) = doc.id
            {
                let _ = state
                    .action_tx
                    .try_send(AppAction::GenerateCitationGraph { doc_ids: vec![id] });
            }
        }
        KeyCode::Char('G') => {
            if let Some(ref gs) = state.graph_state {
                let doc_ids = gs.doc_ids.clone();
                let _ = state
                    .action_tx
                    .try_send(AppAction::GenerateCitationGraph { doc_ids });
            }
        }
        KeyCode::Char('C') => {
            if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor)
                && let Some(id) = doc.id
            {
                let _ = state
                    .action_tx
                    .try_send(AppAction::StartManualCitationEntry { doc_id: id });
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
        KeyCode::Char(c) if c.is_ascii_digit() && state.active_panel == PanelFocus::Left => {
            let notation = c.to_string();
            let _ = state
                .action_tx
                .try_send(AppAction::SelectUdc(Some(notation)));
        }
        KeyCode::Char('o') => {
            if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor)
                && let Some(id) = doc.id
            {
                let _ = state
                    .action_tx
                    .try_send(AppAction::OpenExternalViewer { doc_id: id });
            }
        }

        KeyCode::Char('v') => {
            let _ = state.action_tx.try_send(AppAction::SaveCurrentSearch);
        }
        KeyCode::Char('*') => {
            let _ = state.action_tx.try_send(AppAction::ToggleFavoriteFilter);
        }
        KeyCode::Char('Q') => {
            if state.projects.is_empty() {
                state.set_status("프로젝트가 없습니다 (n 키로 생성)");
            } else {
                // 선택된 문서가 없다면, 현재 포커스된 단일 문서를 선택 목록에 추가
                if state.selected_doc_ids.is_empty() {
                    if let Some(doc) = state.documents.get(state.list_cursor) {
                        if let Some(id) = doc.id {
                            state.selected_doc_ids.insert(id);
                        }
                    }
                }

                if state.selected_doc_ids.is_empty() {
                    state.set_status("포커스 또는 선택된 문헌이 없습니다");
                } else {
                    state.pick_project_mode = true;
                    state.pick_project_input.clear();
                    state.pick_project_cursor = 0;
                    state.set_status("프로젝트를 선택 후 Enter");
                    state.dirty = true;
                }
            }
        }
        KeyCode::Char('R') => {
            if let Some(project_id) = state.active_project_id {
                let mut doc_ids = Vec::new();
                if !state.selected_doc_ids.is_empty() {
                    doc_ids.extend(state.selected_doc_ids.iter().copied());
                } else if let Some(doc) = state.documents.get(state.list_cursor) {
                    if let Some(id) = doc.id {
                        doc_ids.push(id);
                    }
                }

                if doc_ids.is_empty() {
                    state.set_status("제거할 문헌이 없습니다");
                } else {
                    let _ = state.action_tx.try_send(AppAction::RemoveDocsFromProject {
                        project_id,
                        doc_ids,
                    });
                }
            } else {
                state.set_status("프로젝트 필터 상태가 아닙니다 (왼쪽 패널에서 프로젝트를 먼저 선택하세요)");
            }
        }
        KeyCode::Char('i') => {
            let _ = state.action_tx.try_send(AppAction::ToggleStatsDashboard);
        }
        KeyCode::Char('I') => {
            let _ = state.action_tx.try_send(AppAction::StartFileImport);
        }
        KeyCode::Char('F') => {
            if state.active_panel == PanelFocus::Right
                && let Some(doc) = state.documents.get(state.list_cursor)
                && let Some(id) = doc.id
            {
                let _ = state
                    .action_tx
                    .try_send(AppAction::FetchForwardCitations { doc_id: id });
            }
        }
        KeyCode::Char('E') => {
            let _ = state.action_tx.try_send(AppAction::StartAuthorMerge);
        }
        _ => {}
    }
    false
}

fn move_cursor_down(state: &mut AppState) {
    match state.active_panel {
        PanelFocus::Right => {
            let len = if state.queue_view {
                state.queue.len()
            } else {
                state.documents.len()
            };
            if state.list_cursor + 1 < len {
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
        && let Some(doc) = state.documents.get(state.list_cursor)
    {
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
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
            state.show_detail = false;
            state.detail_doc = None;
            state.active_panel = PanelFocus::Right;
            state.dirty = true;
        }
        KeyCode::Char('n') => {
            state.note_mode = true;
            state.note_input.clear();
            state.set_status("노트 작성 (Enter: 줄바꿈, Esc: 저장)");
            state.dirty = true;
        }
        KeyCode::Char('c') => {
            state.custom_field_mode = true;
            state.custom_field_key.clear();
            state.custom_field_value.clear();
            state.custom_field_editing_key = true;
            state.set_status("필드 추가 (Tab: 키/값 전환, Enter: 저장)");
            state.dirty = true;
        }
        KeyCode::Char('t') => {
            if state.detail_doc.as_ref().and_then(|d| d.id).is_some() {
                state.tag_mode = true;
                state.tag_input = state.current_tags.join(" ");
                state.set_status("태그 편집 (스페이스 구분, Esc 저장)");
                state.dirty = true;
            }
        }
        KeyCode::Char('r') => {
            if state.detail_doc.as_ref().and_then(|d| d.id).is_some() {
                state.rating_mode = true;
                state.set_status("별점 입력 (1-5, 0=삭제, Esc 취소)");
                state.dirty = true;
            }
        }
        KeyCode::Char('?') => {
            state.show_help = !state.show_help;
            if state.show_help {
                state.help_page = 0;
            }
            state.dirty = true;
        }
        KeyCode::Tab | KeyCode::Right => cycle_panel(state, true),
        KeyCode::Left => {
            state.show_detail = false;
            state.detail_doc = None;
            state.active_panel = PanelFocus::Right;
            state.dirty = true;
        }
        KeyCode::Char('j') | KeyCode::Down => move_cursor_down(state),
        KeyCode::Char('k') | KeyCode::Up => move_cursor_up(state),
        KeyCode::Char('o') => {
            if let Some(id) = state.detail_doc.as_ref().and_then(|d| d.id) {
                let _ = state
                    .action_tx
                    .try_send(AppAction::OpenExternalViewer { doc_id: id });
            }
        }
        KeyCode::Char('u') => {
            if let Some(id) = state.detail_doc.as_ref().and_then(|d| d.id) {
                let _ = state
                    .action_tx
                    .try_send(AppAction::ToggleReadingStatus { doc_id: id });
            }
        }
        KeyCode::Char('b') => {
            if let Some(id) = state.detail_doc.as_ref().and_then(|d| d.id) {
                let _ = state
                    .action_tx
                    .try_send(AppAction::ExtractBookmarks { doc_id: id });
            }
        }
        KeyCode::Char('F') => {
            if let Some(id) = state.detail_doc.as_ref().and_then(|d| d.id) {
                let _ = state
                    .action_tx
                    .try_send(AppAction::FetchForwardCitations { doc_id: id });
            }
        }
        _ => {}
    }
    false
}

fn handle_note_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            let doc_id = state.detail_doc.as_ref().and_then(|d| d.id).unwrap_or(0);
            let content = state.note_input.clone();
            if !content.is_empty() {
                let notes = state.db.lock().ok().and_then(|conn| {
                    if let Err(e) = crate::db::notes::create(&conn, doc_id, &content, "general") {
                        tracing::error!(error = %e, doc_id, "note_key: notes::create failed");
                        return None;
                    }
                    crate::db::notes::list(&conn, doc_id).ok()
                });
                match notes {
                    Some(list) => state.current_notes = list,
                    None => state.set_status("노트 저장 실패"),
                }
            }
            state.note_mode = false;
            state.note_input.clear();
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

fn handle_tag_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            let doc_id = state.detail_doc.as_ref().and_then(|d| d.id).unwrap_or(0);
            let new_tags: Vec<String> = state
                .tag_input
                .split_whitespace()
                .map(|s| s.trim_matches('#').to_string())
                .filter(|s| !s.is_empty())
                .collect();
            {
                if let Ok(conn) = state.db.lock() {
                    let old_tags = crate::db::documents::get_tags(&conn, doc_id).unwrap_or_default();
                    for tag in &old_tags {
                        if !new_tags.contains(tag) {
                            if let Err(e) = crate::db::documents::remove_tag(&conn, doc_id, tag) {
                                tracing::error!(error = %e, doc_id, tag = %tag, "tag_key: remove_tag failed");
                            }
                        }
                    }
                    for tag in &new_tags {
                        if !old_tags.contains(tag) {
                            if let Err(e) = crate::db::documents::add_tag(&conn, doc_id, tag) {
                                tracing::error!(error = %e, doc_id, tag = %tag, "tag_key: add_tag failed");
                            }
                        }
                    }
                }
            }
            state.current_tags = new_tags;
            state.tag_mode = false;
            state.tag_input.clear();
            state.set_status("태그 저장됨");
            state.dirty = true;
        }
        KeyCode::Backspace => {
            state.tag_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.tag_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_rating_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.rating_mode = false;
            state.set_status("준비됨");
            state.dirty = true;
        }
        KeyCode::Char(c @ '1'..='5') => {
            let rating = c.to_digit(10).unwrap() as u8;
            let doc_id = state.detail_doc.as_ref().and_then(|d| d.id).unwrap_or(0);
            let _ = state.action_tx.try_send(AppAction::SetRating {
                doc_id,
                rating: Some(rating),
            });
            state.rating_mode = false;
            state.dirty = true;
        }
        KeyCode::Char('0') | KeyCode::Backspace => {
            let doc_id = state.detail_doc.as_ref().and_then(|d| d.id).unwrap_or(0);
            let _ = state.action_tx.try_send(AppAction::SetRating {
                doc_id,
                rating: None,
            });
            state.rating_mode = false;
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_save_search_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.save_search_mode = false;
            state.save_search_input.clear();
            state.set_status("취소됨");
            state.dirty = true;
        }
        KeyCode::Enter => {
            let name = state.save_search_input.trim().to_string();
            if name.is_empty() {
                state.set_status("이름을 입력하세요");
            } else {
                let _ = state
                    .action_tx
                    .try_send(AppAction::SaveCurrentSearchNamed { name });
            }
        }
        KeyCode::Backspace => {
            state.save_search_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.save_search_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_bulk_import_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.bulk_import_mode = false;
            state.bulk_import_input.clear();
            state.set_status("취소됨");
            state.dirty = true;
        }
        KeyCode::Enter => {
            let input = state.bulk_import_input.clone();
            let _ = state
                .action_tx
                .try_send(AppAction::BulkImportSubmitted(input));
        }
        KeyCode::Backspace => {
            state.bulk_import_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.bulk_import_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_file_import_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.file_import_mode = false;
            state.file_import_input.clear();
            state.set_status("취소됨");
            state.dirty = true;
        }
        KeyCode::Enter => {
            let path = state.file_import_input.trim().to_string();
            if path.is_empty() {
                state.set_status("파일 경로를 입력하세요");
            } else {
                let _ = state
                    .action_tx
                    .try_send(AppAction::FileImportSubmitted(path));
            }
        }
        KeyCode::Backspace => {
            state.file_import_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.file_import_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_author_merge_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.author_merge_mode = false;
            state.author_merge_phase = 0;
            state.author_merge_source.clear();
            state.author_merge_input.clear();
            state.set_status("취소됨");
            state.dirty = true;
        }
        KeyCode::Enter => {
            let input = state.author_merge_input.trim().to_string();
            if input.is_empty() {
                state.set_status("저자명을 입력하세요");
                return false;
            }
            if state.author_merge_phase == 1 {
                let _ = state
                    .action_tx
                    .try_send(AppAction::AuthorMergeSourceEntered(input));
            } else {
                let source = state.author_merge_source.clone();
                let _ = state
                    .action_tx
                    .try_send(AppAction::AuthorMergeCanonicalEntered {
                        source,
                        canonical: input,
                    });
            }
        }
        KeyCode::Backspace => {
            state.author_merge_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.author_merge_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_stats_overlay_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('i') | KeyCode::Char('q') => {
            let _ = state.action_tx.try_send(AppAction::ToggleStatsDashboard);
        }
        _ => {
            let _ = state.action_tx.try_send(AppAction::ToggleStatsDashboard);
        }
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
            if state.search_input.is_empty() {
                state.reload_documents();
            }
        }
        KeyCode::Backspace => {
            state.search_input.pop();
            let term = state.search_input.clone();
            handle_search_filter(state, term);
        }
        KeyCode::Char(c) => {
            state.search_input.push(c);
            let term = state.search_input.clone();
            handle_search_filter(state, term);
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

fn handle_new_series_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.new_series_mode = false;
            state.new_series_input.clear();
            state.set_status("준비됨");
        }
        KeyCode::Enter => {
            state.new_series_mode = false;
            let name = state.new_series_input.clone();
            state.new_series_input.clear();
            if !name.is_empty() {
                let _ = state.action_tx.try_send(AppAction::CreateSeries(name));
            } else {
                state.set_status("준비됨");
            }
        }
        KeyCode::Backspace => {
            state.new_series_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.new_series_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn filtered_projects(state: &AppState) -> Vec<&Project> {
    let q = state.pick_project_input.to_lowercase();
    state
        .projects
        .iter()
        .filter(|p| q.is_empty() || p.name.to_lowercase().contains(&q))
        .collect()
}

pub fn filtered_authors(state: &AppState) -> Vec<&(String, i64)> {
    let q = state.author_search_input.to_lowercase();
    state
        .authors
        .iter()
        .filter(|(name, _)| q.is_empty() || name.to_lowercase().contains(&q))
        .collect()
}

fn handle_pick_project_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.pick_project_mode = false;
            state.pick_project_input.clear();
            state.pick_project_cursor = 0;
            state.set_status("준비됨");
        }
        KeyCode::Enter => {
            let filtered = filtered_projects(state);
            if let Some(proj) = filtered.get(state.pick_project_cursor) {
                let project_id = proj.id.unwrap_or(0);
                let doc_ids: Vec<i64> = state.selected_doc_ids.iter().copied().collect();
                state.pick_project_mode = false;
                state.pick_project_input.clear();
                state.pick_project_cursor = 0;
                let _ = state.action_tx.try_send(AppAction::AddDocsToProject {
                    project_id,
                    doc_ids,
                });
            } else {
                state.set_status("선택할 프로젝트가 없습니다");
            }
        }
        KeyCode::Backspace => {
            state.pick_project_input.pop();
            let new_len = filtered_projects(state).len();
            if state.pick_project_cursor >= new_len && new_len > 0 {
                state.pick_project_cursor = new_len - 1;
            } else if new_len == 0 {
                state.pick_project_cursor = 0;
            }
            state.dirty = true;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let filtered_len = filtered_projects(state).len();
            if filtered_len > 0 {
                state.pick_project_cursor = (state.pick_project_cursor + 1) % filtered_len;
                state.dirty = true;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let filtered_len = filtered_projects(state).len();
            if filtered_len > 0 {
                if state.pick_project_cursor == 0 {
                    state.pick_project_cursor = filtered_len - 1;
                } else {
                    state.pick_project_cursor -= 1;
                }
                state.dirty = true;
            }
        }
        KeyCode::Char(c) => {
            state.pick_project_input.push(c);
            let new_len = filtered_projects(state).len();
            if state.pick_project_cursor >= new_len && new_len > 0 {
                state.pick_project_cursor = new_len - 1;
            } else if new_len == 0 {
                state.pick_project_cursor = 0;
            }
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_author_search_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.author_search_mode = false;
            state.author_search_input.clear();
            state.set_status("준비됨");
        }
        KeyCode::Enter => {
            state.author_search_mode = false;
            state.set_status("검색 완료");
        }
        KeyCode::Backspace => {
            state.author_search_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.author_search_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_confirm_delete_key(state: &mut AppState, key: KeyEvent) -> bool {
    let key = normalize_korean_key(key);
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            if let Some(id) = state.delete_confirm_doc_id.take() {
                let _ = state.action_tx.try_send(AppAction::DeleteDocument(id));
            }
            state.confirm_delete_mode = false;
            state.delete_confirm_title.clear();
            state.dirty = true;
        }
        KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => {
            state.confirm_delete_mode = false;
            state.delete_confirm_doc_id = None;
            state.delete_confirm_title.clear();
            state.set_status("삭제 취소됨");
        }
        KeyCode::Char('s') => {
            state.skip_delete_confirm = true;
            if let Ok(conn) = state.db.lock() {
                if let Err(e) = conn.execute(
                    "INSERT OR REPLACE INTO app_config (key, value, updated_at) \
                     VALUES ('skip_delete_confirm', 'true', datetime('now'))",
                    [],
                ) {
                    tracing::error!("Failed to save skip_delete_confirm setting: {e}");
                }
            }
            if let Some(id) = state.delete_confirm_doc_id.take() {
                let _ = state.action_tx.try_send(AppAction::DeleteDocument(id));
            }
            state.confirm_delete_mode = false;
            state.delete_confirm_title.clear();
            state.set_status("앞으로 삭제 시 확인하지 않습니다");
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
                    let _ = state
                        .action_tx
                        .try_send(AppAction::UpdateDocument(id, Box::new(doc)));
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
        9 => doc.item_type.clone(),
        _ => String::new(),
    }
}

fn apply_edit_to_doc(doc: &mut documents::Document, field: usize, value: &str) {
    let trimmed = value.trim().to_string();
    match field {
        0 => doc.title = trimmed,
        1 => {
            doc.authors = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        2 => {
            doc.journal = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        3 => {
            doc.conference = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        4 => doc.pub_year = trimmed.parse::<i64>().ok(),
        5 => {
            doc.doi = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        6 => {
            doc.arxiv_id = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        7 => {
            doc.abstract_text = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        8 => {
            doc.keywords = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        9 => {
            doc.item_type = if trimmed.is_empty() {
                "misc".to_string()
            } else {
                trimmed
            }
        }
        _ => {}
    }
}

fn handle_tree_activate(state: &mut AppState) {
    let cursor = state.tree_cursor;
    let projects_header = 1usize;
    let projects_rows = state.projects.len().max(1);
    let first_spacer = 1usize;

    if cursor < projects_header + projects_rows + first_spacer {
        if cursor >= projects_header && cursor < projects_header + projects_rows {
            let proj_idx = cursor - projects_header;
            if let Some(proj) = state.projects.get(proj_idx) {
                let new_id = if state.active_project_id == proj.id {
                    None
                } else {
                    proj.id
                };
                let _ = state.action_tx.try_send(AppAction::SelectProject(new_id));
            }
        }
        return;
    }

    let mut idx = projects_header + projects_rows + first_spacer;

    if state.series_grouping_enabled {
        let series_header = 1usize;
        let series_rows = state.series.len().max(1);
        let series_section = series_header + series_rows;
        if cursor < idx + series_section + 1 {
            if cursor >= idx + series_header && cursor < idx + series_header + series_rows {
                let ser_idx = cursor - (idx + series_header);
                if let Some(ser) = state.series.get(ser_idx) {
                    let new_id = if state.active_series_id == ser.id {
                        None
                    } else {
                        ser.id
                    };
                    let _ = state.action_tx.try_send(AppAction::SelectSeries(new_id));
                }
            }
            return;
        }
        idx += series_section + 1; // +1 for trailing spacer
    }

    let authors_header = 1usize;
    if !state.authors.is_empty() {
        let filtered = filtered_authors(state);
        let authors_rows = if state.authors_expanded {
            filtered.len()
        } else {
            0
        };
        let authors_section = authors_header + authors_rows;
        if cursor < idx + authors_section + 1 {
            if cursor == idx {
                state.authors_expanded = !state.authors_expanded;
                state.author_search_input.clear();
                state.dirty = true;
                return;
            }
            if state.authors_expanded
                && cursor >= idx + authors_header
                && cursor < idx + authors_header + authors_rows
            {
                let auth_idx = cursor - (idx + authors_header);
                if let Some((name, _)) = filtered.get(auth_idx) {
                    let new_name = if state.active_author.as_deref() == Some(name.as_str()) {
                        None
                    } else {
                        Some(name.clone())
                    };
                    let _ = state.action_tx.try_send(AppAction::SelectAuthor(new_name));
                }
            }
            return;
        }
        idx += authors_section + 1; // +1 for trailing spacer
    }

    let tree_idx = cursor - idx - 1; // -1 to skip the UDC header
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
        || state.new_series_mode
        || state.pick_project_mode
        || state.author_search_mode
        || state.citation_entry_mode
        || state.bibtex_import_mode
        || state.note_mode
        || state.tag_mode
        || state.rating_mode
        || state.confirm_delete_mode
        || state.api_key_input_mode
        || state.custom_field_mode
        || state.show_help
        || state.show_export_dialog
        || state.settings_panel_mode
        || state.show_widget_panel
        || state.queue_view
}

fn compute_body_rect(state: &AppState) -> Rect {
    let (w, h) = state.terminal_size;
    let area = Rect {
        x: 0,
        y: 0,
        width: w,
        height: h,
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);
    chunks[1]
}

fn compute_right_panel_rect(state: &AppState) -> Rect {
    let body = compute_body_rect(state);

    if state.show_detail {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(42),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(body);
        split[0]
    } else {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(state.left_panel_width),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
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
        .constraints([
            Constraint::Length(state.left_panel_width),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
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
        && is_in_rect(column, row, &left_rect)
    {
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
            if let Some(doc) = state.documents.get(item_index) {
                let id = doc.id.unwrap_or(0);
                if state.selected_doc_ids.contains(&id) {
                    state.selected_doc_ids.remove(&id);
                } else {
                    state.selected_doc_ids.insert(id);
                }
            }
            state.dirty = true;
        }
    } else if let Some(left_rect) = left_rect
        && is_in_rect(column, row, &left_rect)
    {
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
    column >= rect.x && column < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

fn handle_drag(state: &mut AppState, path: PathBuf) {
    state.start_processing(&format!("처리 중: {}", path.display()));

    let tx = state.action_tx.clone();
    let path_clone = path.clone();

    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || pdf::process_file(&path_clone)).await;

        match result {
            Ok(Ok(meta)) => {
                let _ = tx
                    .send(AppAction::MetadataExtracted(Box::new(meta), path))
                    .await;
            }
            Ok(Err(e)) => {
                let _ = tx.send(AppAction::OperationFailed(e.to_string())).await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppAction::OperationFailed(format!("태스크 실패: {}", e)))
                    .await;
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
        && let Ok(Some(_)) = documents::find_by_hash(conn, h)
    {
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
        rating: None,
        ..Default::default()
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
    state.active_author = None;
    state.active_udc_notation = None;
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

fn handle_delete_project(state: &mut AppState, project_id: i64) {
    let result = {
        if let Ok(conn) = state.db.lock() {
            crate::db::projects::delete_project(&conn, project_id)
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(_) => {
            if state.active_project_id == Some(project_id) {
                state.active_project_id = None;
                state.reload_documents();
            }
            state.reload_projects();
            state.set_status("프로젝트 삭제됨");
        }
        Err(e) => state.set_status(&format!("프로젝트 삭제 실패: {}", e)),
    }
}

fn handle_add_docs_to_project(state: &mut AppState, project_id: i64, doc_ids: Vec<i64>) {
    let project_name = state
        .projects
        .iter()
        .find(|p| p.id == Some(project_id))
        .map(|p| p.name.as_str())
        .unwrap_or("프로젝트")
        .to_string();
    let result = {
        if let Ok(conn) = state.db.lock() {
            for doc_id in &doc_ids {
                if let Err(e) = crate::db::projects::add_document(&conn, project_id, *doc_id) {
                    tracing::error!("Failed to add doc {doc_id} to project {project_id}: {e}");
                }
            }
            Ok::<(), anyhow::Error>(())
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(_) => {
            state.selected_doc_ids.clear();
            state.set_status(&format!(
                "{}건 문헌을 '{}'에 추가",
                doc_ids.len(),
                project_name
            ));
            if state.active_project_id == Some(project_id) {
                handle_select_project(state, Some(project_id));
            }
            state.dirty = true;
        }
        Err(e) => state.set_status(&format!("추가 실패: {}", e)),
    }
}

fn handle_remove_docs_from_project(state: &mut AppState, project_id: i64, doc_ids: Vec<i64>) {
    let project_name = state
        .projects
        .iter()
        .find(|p| p.id == Some(project_id))
        .map(|p| p.name.as_str())
        .unwrap_or("프로젝트")
        .to_string();
    let result = {
        if let Ok(conn) = state.db.lock() {
            for doc_id in &doc_ids {
                if let Err(e) = crate::db::projects::remove_document(&conn, project_id, *doc_id) {
                    tracing::error!("Failed to remove doc {doc_id} from project {project_id}: {e}");
                }
            }
            Ok::<(), anyhow::Error>(())
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(_) => {
            state.selected_doc_ids.clear();
            state.set_status(&format!(
                "{}건 문헌을 '{}'에서 제거",
                doc_ids.len(),
                project_name
            ));
            if state.active_project_id == Some(project_id) {
                handle_select_project(state, Some(project_id));
            }
            state.dirty = true;
        }
        Err(e) => state.set_status(&format!("제거 실패: {}", e)),
    }
}

fn handle_create_series(state: &mut AppState, name: String) {
    let result = {
        if let Ok(conn) = state.db.lock() {
            crate::db::series::create_series(&conn, &name, None, None)
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(_id) => {
            if !state.series_grouping_enabled {
                state.series_grouping_enabled = true;
                    if let Ok(conn) = state.db.lock() {
                        if let Err(e) = conn.execute(
                            "INSERT OR REPLACE INTO app_config (key, value, updated_at) VALUES ('series_grouping_enabled', 'true', datetime('now'))",
                            [],
                        ) {
                            tracing::error!("Failed to save series_grouping_enabled setting: {e}");
                        }
                    }
                }
                state.reload_series();
                state.set_status(&format!("시리즈 생성: {}", name));
        }
        Err(e) => state.set_status(&format!("시리즈 생성 실패: {}", e)),
    }
}

fn handle_select_series(state: &mut AppState, series_id: Option<i64>) {
    state.active_series_id = series_id;
    state.active_author = None;
    state.active_udc_notation = None;
    if let Some(sid) = series_id {
        let docs = {
            if let Ok(conn) = state.db.lock() {
                if let Ok(ids) = crate::db::series::list_documents(&conn, sid) {
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

fn handle_select_author(state: &mut AppState, author: Option<String>) {
    state.active_author = author.clone();
    state.active_project_id = None;
    state.active_series_id = None;
    state.active_udc_notation = None;
    if let Some(ref name) = author {
        let conn = state.db.lock();
        if let Ok(conn) = conn {
            let all = documents::list_all(&conn).unwrap_or_default();
            let norm_name = crate::db::fts_query::normalize_nfc(name);
            let filtered: Vec<Document> = all
                .into_iter()
                .filter(|d| {
                    d.authors
                        .as_deref()
                        .map(|a| {
                            documents::split_authors(a)
                                .iter()
                                .any(|seg| crate::db::fts_query::normalize_nfc(seg) == norm_name)
                        })
                        .unwrap_or(false)
                })
                .collect();
            let count = filtered.len();
            state.documents = filtered;
            state.document_count = count;
            state.list_cursor = 0;

            if state.auto_fetch_metrics
                && state.api_mode.allows_api_calls()
                && !state.author_metrics.contains_key(name)
            {
                let backend = state.metrics_backend;
                let max_age = state.metrics_refresh_interval_days;
                let has_fresh =
                    crate::api::metrics::get_cached_metrics(&conn, backend, name, max_age)
                        .ok()
                        .flatten()
                        .is_some();
                if !has_fresh {
                    let name_for_fetch = name.clone();
                    let _ = state.action_tx.try_send(AppAction::FetchAuthorMetrics {
                        name: name_for_fetch,
                    });
                }
            }
        }
    } else {
        state.reload_documents();
    }
    state.dirty = true;
}

fn handle_delete_series(state: &mut AppState, series_id: i64) {
    let result = {
        if let Ok(conn) = state.db.lock() {
            crate::db::series::delete_series(&conn, series_id)
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(_) => {
            if state.active_series_id == Some(series_id) {
                state.active_series_id = None;
                state.reload_documents();
            }
            state.reload_series();
            state.set_status("시리즈 삭제됨");
        }
        Err(e) => state.set_status(&format!("시리즈 삭제 실패: {}", e)),
    }
}

fn handle_toggle_series_grouping(state: &mut AppState) {
    state.series_grouping_enabled = !state.series_grouping_enabled;
    let value = if state.series_grouping_enabled {
        "true"
    } else {
        "false"
    };
    if let Ok(conn) = state.db.lock() {
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO app_config (key, value, updated_at) VALUES ('series_grouping_enabled', ?1, datetime('now'))",
            rusqlite::params![value],
        ) {
            tracing::error!("Failed to save series_grouping_enabled setting: {e}");
        }
    }
    if !state.series_grouping_enabled {
        state.active_series_id = None;
        state.reload_documents();
    } else {
        state.reload_series();
    }
    state.set_status(if state.series_grouping_enabled {
        "시리즈 그룹핑 활성화"
    } else {
        "시리즈 그룹핑 비활성화"
    });
    state.dirty = true;
}

fn handle_assign_doc_to_series(
    state: &mut AppState,
    doc_id: i64,
    series_id: i64,
    volume: Option<String>,
    issue: Option<String>,
) {
    let result = {
        if let Ok(conn) = state.db.lock() {
            crate::db::series::add_document(
                &conn,
                series_id,
                doc_id,
                volume.as_deref(),
                issue.as_deref(),
            )
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(_) => {
            state.set_status(&format!("문헌 {}을(를) 시리즈에 추가", doc_id));
            state.dirty = true;
        }
        Err(e) => state.set_status(&format!("시리즈 추가 실패: {}", e)),
    }
}

fn handle_auto_group_series(state: &mut AppState) {
    let result = {
        if let Ok(conn) = state.db.lock() {
            crate::db::series::auto_group_by_journal(&conn)
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(ids) => {
            if !ids.is_empty() {
                if !state.series_grouping_enabled {
                    state.series_grouping_enabled = true;
                        if let Ok(conn) = state.db.lock() {
                            if let Err(e) = conn.execute(
                                "INSERT OR REPLACE INTO app_config (key, value, updated_at) VALUES ('series_grouping_enabled', 'true', datetime('now'))",
                                [],
                            ) {
                                tracing::error!("Failed to save series_grouping_enabled setting: {e}");
                            }
                        }
                    }
                    state.reload_series();
                    state.set_status(&format!("자동 그룹핑: {}개 시리즈 생성/갱신", ids.len()));
            } else {
                state.set_status("자동 그룹핑: 묶을 수 있는 시리즈 없음");
            }
        }
        Err(e) => state.set_status(&format!("자동 그룹핑 실패: {}", e)),
    }
}

fn handle_export(state: &mut AppState, format: ExportFormat) {
    let scope = state.export_dialog_state.selected_scope;
    let backup_scope = state.export_dialog_state.selected_backup_scope;

    let export_result = {
        if let Ok(conn) = state.db.lock() {
            let home = directories::BaseDirs::new()
                .map(|d| d.home_dir().to_path_buf())
                .unwrap_or_else(|| std::path::PathBuf::from("."));

            if scope == crate::export::export_dialog_state::ExportScope::BackupDb {
                let temp_path = std::env::temp_dir().join("libran_temp_backup.db");
                let _ = std::fs::remove_file(&temp_path);

                if let Err(e) = crate::db::backup::backup_to_path(&conn, &temp_path) {
                    Some(Err(format!("임시 백업 파일 생성 실패: {}", e)))
                } else {
                    let process_res = (|| -> Result<(), anyhow::Error> {
                        let temp_conn = rusqlite::Connection::open(&temp_path)?;
                        match backup_scope {
                            crate::export::export_dialog_state::BackupScope::FullMigration => {}
                            crate::export::export_dialog_state::BackupScope::BackupWithoutApiKeys => {
                                temp_conn.execute("DELETE FROM app_config WHERE key = 'openalex_api_key'", [])?;
                            }
                            crate::export::export_dialog_state::BackupScope::DocsOnly => {
                                temp_conn.execute("DELETE FROM document_notes", [])?;
                                temp_conn.execute("UPDATE documents SET reading_progress = NULL, reading_status = 'unread'", [])?;
                            }
                        }
                        temp_conn.execute("VACUUM", [])?;
                        Ok(())
                    })();

                    if let Err(e) = process_res {
                        let _ = std::fs::remove_file(&temp_path);
                        Some(Err(format!("백업 데이터 가공 실패: {}", e)))
                    } else {
                        let filename = match backup_scope {
                            crate::export::export_dialog_state::BackupScope::FullMigration => "libran_migration.db",
                            crate::export::export_dialog_state::BackupScope::BackupWithoutApiKeys => "libran_shared_backup.db",
                            crate::export::export_dialog_state::BackupScope::DocsOnly => "libran_docs_only.db",
                        };
                        let export_path = home.join(filename);
                        if let Err(e) = std::fs::copy(&temp_path, &export_path) {
                            let _ = std::fs::remove_file(&temp_path);
                            Some(Err(format!("백업 파일 저장 실패: {}", e)))
                        } else {
                            let _ = std::fs::remove_file(&temp_path);
                            Some(Ok(format!(
                                "DB 백업 내보내기 완료: {}",
                                export_path.display()
                            )))
                        }
                    }
                }
            } else {
                let docs: Vec<documents::Document> = match scope {
                    crate::export::export_dialog_state::ExportScope::SelectedOnly => {
                        state.selected_doc_ids
                            .iter()
                            .filter_map(|id| documents::get_by_id(&conn, *id).ok().flatten())
                            .collect()
                    }
                    crate::export::export_dialog_state::ExportScope::EntireLibrary => {
                        documents::list_all(&conn).unwrap_or_default()
                    }
                    _ => Vec::new(),
                };

                if docs.is_empty() {
                    Some(Err("내보낼 문헌을 찾을 수 없습니다 (선택된 문헌이 비어 있을 수 있습니다)".to_string()))
                } else {
                    let filename = format!("export.{}", format.file_extension());
                    let export_path = home.join(filename);

                    match std::fs::File::create(&export_path) {
                        Ok(mut file) => match export(&docs, format, &mut file) {
                            Ok(()) => Some(Ok(format!(
                                "인용 정보 내보내기 완료: {} ({}건)",
                                export_path.display(),
                                docs.len()
                            ))),
                            Err(e) => Some(Err(format!("내보내기 실패: {}", e))),
                        },
                        Err(e) => Some(Err(format!("파일 생성 실패: {}", e))),
                    }
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

fn handle_export_dialog_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.show_export_dialog = false;
            state.dirty = true;
        }
        KeyCode::Tab => {
            state.export_dialog_state.tab_next();
            update_dialog_preview(state);
            state.dirty = true;
        }
        KeyCode::BackTab => {
            state.export_dialog_state.tab_prev();
            update_dialog_preview(state);
            state.dirty = true;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            state.export_dialog_state.cursor_down();
            update_dialog_preview(state);
            state.dirty = true;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.export_dialog_state.cursor_up();
            update_dialog_preview(state);
            state.dirty = true;
        }
        KeyCode::Enter => {
            save_export_preferences(state);
            handle_clipboard_copy(state);
        }
        KeyCode::Char('e') => {
            let format = state.export_dialog_state.selected_format.clone();
            save_export_preferences(state);
            state.show_export_dialog = false;
            handle_export(state, format);
        }
        _ => {}
    }
    false
}

fn update_dialog_preview(state: &mut AppState) {
    if let Ok(conn) = state.db.lock() {
        if let Some(&id) = state.selected_doc_ids.iter().next() {
            if let Ok(Some(doc)) = documents::get_by_id(&conn, id) {
                state.export_dialog_state.update_preview(&doc);
            }
        }
    }
}

fn save_export_preferences(state: &mut AppState) {
    if let Ok(conn) = state.db.lock() {
        let _ = crate::export::preferences::save(
            &conn,
            state.export_dialog_state.selected_format.clone(),
            state.export_dialog_state.selected_style,
            state.export_dialog_state.selected_language,
        );
    }
}

fn handle_clipboard_copy(state: &mut AppState) {
    use crate::citation::text::render_citation;

    let result = {
        if let Ok(conn) = state.db.lock() {
            let docs: Vec<Document> = state
                .selected_doc_ids
                .iter()
                .filter_map(|id| documents::get_by_id(&conn, *id).ok().flatten())
                .collect();

            if docs.is_empty() {
                None
            } else {
                let style = state.export_dialog_state.selected_style;
                let lang = state.export_dialog_state.selected_language;
                let mode = state.export_dialog_state.display_mode;

                let mut text = String::new();
                for doc in &docs {
                    if let Ok(citation) = render_citation(doc, style, lang, mode) {
                        text.push_str(&citation);
                        text.push('\n');
                    }
                }
                Some((text, docs.len()))
            }
        } else {
            None
        }
    };

    match result {
        None => state.set_status("내보낼 문헌을 찾을 수 없습니다"),
        Some((text, count)) => match copy_to_clipboard(&text) {
            Ok(()) => state.set_status(&format!("✓ 클립보드에 복사됨 ({}건)", count)),
            Err(_) => {
                let home = directories::BaseDirs::new()
                    .map(|d| d.home_dir().to_path_buf())
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                let libran_dir = home.join(".libran");
                let _ = std::fs::create_dir_all(&libran_dir);
                let path = libran_dir.join("clipboard.txt");
                match std::fs::write(&path, &text) {
                    Ok(()) => {
                        state.set_status(&format!("클립보드 실패, 파일에 저장: {}", path.display()))
                    }
                    Err(e) => state.set_status(&format!("복사 실패: {}", e)),
                }
            }
        },
    }
}

fn copy_to_clipboard(text: &str) -> anyhow::Result<()> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| anyhow::anyhow!("Clipboard init failed: {}", e))?;
    clipboard
        .set_text(text)
        .map_err(|e| anyhow::anyhow!("Clipboard set failed: {}", e))?;
    Ok(())
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

    let scores = compute_scores(
        &ref_features,
        &features,
        &state.udc_tree,
        &state.similarity_config,
    );

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
    let rows = match stmt.query_map(rusqlite::params![document_id], |row| {
        row.get::<_, String>(0)
    }) {
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
    let mut count = 1; // "프로젝트" header
    count += state.projects.len().max(1);
    count += 1; // spacer after projects
    if state.series_grouping_enabled {
        count += 1; // "시리즈" header
        count += state.series.len().max(1);
        count += 1; // spacer after series
    }
    if !state.authors.is_empty() {
        count += 1; // "연구자별 보기" header
        if state.authors_expanded {
            if state.author_search_mode {
                count += 1; // search input line
            }
            let filtered_len = filtered_authors(state).len();
            if filtered_len == 0 && !state.author_search_input.is_empty() {
                count += 1; // "일치하는 연구자가 없습니다"
            } else {
                count += filtered_len;
            }
        }
        count += 1; // spacer after authors
    }
    count += 1; // "UDC 분류" header
    count += UDC_TOP_LEVEL_STRS.len();
    for (notation, _) in UDC_TOP_LEVEL_TUPLES {
        if state.expanded_nodes.contains(*notation) {
            if let Some(children) = crate::ui::left_panel::UDC_CHILDREN.get(*notation) {
                count += children.len();
            }
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

pub const UDC_TOP_LEVEL_STRS: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

pub const UDC_TOP_LEVEL_TUPLES: &[(&str, &str)] = &[
    ("0", "총류"),
    ("1", "철학"),
    ("2", "종교"),
    ("3", "사회과학"),
    ("4", "(Vacant)"),
    ("5", "자연과학"),
    ("6", "응용과학"),
    ("7", "예술"),
    ("8", "언어"),
    ("9", "역사"),
];

pub fn count_authors_section_start(state: &AppState) -> usize {
    let mut count = 1; // "프로젝트" header
    count += state.projects.len().max(1);
    count += 1; // spacer
    if state.series_grouping_enabled {
        count += 1; // "시리즈" header
        count += state.series.len().max(1);
        count += 1; // spacer
    }
    count
}

fn handle_start_citation_extraction(state: &mut AppState, doc_id: i64) {
    state.start_processing(&format!("인용 추출 중 (doc {})...", doc_id));
    let db = state.db.clone();
    let tx = state.action_tx.clone();

    tokio::spawn(async move {
        let result =
            tokio::task::spawn_blocking(move || -> std::result::Result<(usize, usize), String> {
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
                    if let Err(e) = documents::save_reference_extraction(
                        &conn,
                        doc_id,
                        &text,
                        "heuristic_regex",
                        0,
                    ) {
                        tracing::error!("Failed to save empty reference extraction for doc {doc_id}: {e}");
                    }
                    return Ok((0, 0));
                }

                let section_text: String = refs
                    .iter()
                    .map(|r| r.raw_text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Err(e) = documents::save_reference_extraction(
                    &conn,
                    doc_id,
                    &section_text,
                    "heuristic_regex",
                    2,
                ) {
                    tracing::error!("Failed to save reference extraction for doc {doc_id}: {e}");
                }

                let fuzzy_threshold = 0.85;
                let mut edge_count = 0usize;
                let mut unmatched_count = 0usize;

                for r in &refs {
                    match match_refs::match_reference_to_doc(&conn, r, fuzzy_threshold) {
                        Ok(Some(mr)) => {
                            if let Err(e) = entry::add_extracted_citation(
                                &conn,
                                doc_id,
                                mr.doc_id,
                                &mr.match_status,
                                mr.confidence,
                                Some(&r.raw_text),
                            ) {
                                tracing::error!("Failed to save extracted citation for doc {doc_id}: {e}");
                            }
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
            })
            .await;

        match result {
            Ok(Ok((edge_count, unmatched_count))) => {
                let _ = tx
                    .send(AppAction::CitationExtracted {
                        doc_id,
                        edge_count,
                        unmatched_count,
                    })
                    .await;
            }
            Ok(Err(reason)) => {
                let _ = tx
                    .send(AppAction::CitationExtractionFailed { doc_id, reason })
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppAction::CitationExtractionFailed {
                        doc_id,
                        reason: e.to_string(),
                    })
                    .await;
            }
        }
    });
}

fn handle_open_external(state: &mut AppState, doc_id: i64) {
    let viewer_command = state.config.viewer_command.clone();

    let is_empty_or_none = viewer_command.as_ref().map_or(true, |v| v.is_empty());
    let mut is_sioyek_requested = is_empty_or_none;
    let mut is_okular_requested = false;

    if !is_empty_or_none {
        if let Some(parts) = &viewer_command {
            if let Some(first) = parts.first() {
                let lower = first.to_lowercase();
                if lower.contains("sioyek") {
                    is_sioyek_requested = true;
                } else if lower.contains("okular") {
                    is_okular_requested = true;
                }
            }
        }
    }

    // Sioyek이 필요한데 로컬 시스템에 감지되지 않는 경우 설치 질문 다이얼로그 실행
    if is_sioyek_requested && crate::pdf::sioyek::find_sioyek_executable().is_none() {
        state.show_sioyek_install_dialog = true;
        state.sioyek_install_doc_id = Some(doc_id);
        state.finish_processing("Sioyek 설치 필요");
        state.dirty = true;
        return;
    }

    // Okular가 필요한데 로컬 시스템에 감지되지 않는 경우 설치 질문 다이얼로그 실행
    if is_okular_requested && crate::pdf::okular::find_okular_executable().is_none() {
        state.show_okular_install_dialog = true;
        state.okular_install_doc_id = Some(doc_id);
        state.finish_processing("Okular 설치 필요");
        state.dirty = true;
        return;
    }

    state.start_processing("PDF 여는 중...");
    let db = state.db.clone();
    let tx = state.action_tx.clone();

    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || -> std::result::Result<Option<String>, String> {
            let file_path = {
                let conn = db.lock().map_err(|e| e.to_string())?;
                documents::get_by_id(&conn, doc_id)
                    .map_err(|e| e.to_string())?
                    .and_then(|d| d.file_path)
                    .ok_or_else(|| "파일 경로가 없습니다".to_string())?
            };

            let path = std::path::Path::new(&file_path);
            if !library::check_file_exists(path) {
                return Err(format!("파일을 찾을 수 없습니다: {}", file_path));
            }

            let mut is_sioyek = false;
            let mut exe_name = "sioyek".to_string();
            let mut sioyek_not_found_warning = false;

            let is_empty_or_none = viewer_command.as_ref().map_or(true, |v| v.is_empty());

            if is_empty_or_none {
                // viewer_command가 지정되지 않았거나 빈 벡터인 경우(기본값), Sioyek 실행 파일을 탐색하여 Sioyek으로 실행합니다.
                if let Some(sioyek_path) = crate::pdf::sioyek::find_sioyek_executable() {
                    is_sioyek = true;
                    exe_name = sioyek_path;
                } else {
                    sioyek_not_found_warning = true;
                }
            } else if let Some(parts) = &viewer_command {
                if let Some(first) = parts.first() {
                    let lower = first.to_lowercase();
                    if lower.contains("sioyek") {
                        is_sioyek = true;
                        exe_name = first.clone();
                    }
                }
            }

            if is_sioyek {
                let progress = {
                    let conn = db.lock().map_err(|e| e.to_string())?;
                    documents::get_by_id(&conn, doc_id)
                        .map_err(|e| e.to_string())?
                        .and_then(|d| d.reading_progress)
                        .unwrap_or(0)
                };

                let mut cmd = std::process::Command::new(&exe_name);
                if progress > 0 {
                    // Sioyek CLI --page 인자는 1-indexed이므로 progress(0-indexed)에 1을 더합니다.
                    cmd.arg("--page").arg((progress + 1).to_string());
                }
                // -- marker prevents argument injection if file_path starts with '-'
                cmd.arg("--");
                cmd.arg(&file_path);
                cmd.stdout(std::process::Stdio::null());
                cmd.stderr(std::process::Stdio::null());

                let mut child = cmd.spawn().map_err(|e| format!("Sioyek 실행 실패: {}", e))?;
                let status = child.wait().map_err(|e| format!("Sioyek 대기 오류: {}", e))?;

                if status.success() {
                    let conn = db.lock().map_err(|e| e.to_string())?;
                    let sync_msg = crate::pdf::sioyek::sync_sioyek_data(&conn, doc_id)
                        .map_err(|e| format!("Sioyek 동기화 실패: {}", e))?;
                    return Ok(Some(sync_msg));
                } else {
                    return Err(format!("Sioyek 프로세스가 오류로 종료되었습니다 (코드: {})", status));
                }
            }

            let mut is_okular = false;
            let mut okular_exe = "okular".to_string();
            let mut okular_not_found_warning = false;

            if is_okular_requested {
                if let Some(okular_path) = crate::pdf::okular::find_okular_executable() {
                    is_okular = true;
                    okular_exe = okular_path;
                } else {
                    okular_not_found_warning = true;
                }
            }

            if is_okular {
                let progress = {
                    let conn = db.lock().map_err(|e| e.to_string())?;
                    documents::get_by_id(&conn, doc_id)
                        .map_err(|e| e.to_string())?
                        .and_then(|d| d.reading_progress)
                        .unwrap_or(0)
                };

                let mut cmd = std::process::Command::new(&okular_exe);
                if progress > 0 {
                    cmd.arg("--page").arg((progress + 1).to_string());
                }
                // -- marker prevents argument injection if file_path starts with '-'
                cmd.arg("--");
                cmd.arg(&file_path);
                cmd.stdout(std::process::Stdio::null());
                cmd.stderr(std::process::Stdio::null());

                let mut child = cmd.spawn().map_err(|e| format!("Okular 실행 실패: {}", e))?;
                let status = child.wait().map_err(|e| format!("Okular 대기 오류: {}", e))?;

                if status.success() {
                    let conn = db.lock().map_err(|e| e.to_string())?;
                    let sync_msg = crate::pdf::okular::sync_okular_data(&conn, doc_id)
                        .map_err(|e| format!("Okular 동기화 실패: {}", e))?;
                    return Ok(Some(sync_msg));
                } else {
                    return Err(format!("Okular 프로세스가 오류로 종료되었습니다 (코드: {})", status));
                }
            }

            // Sioyek/Okular이 아닌 일반 뷰어 실행 로직
            match &viewer_command {
                Some(parts) if !parts.is_empty() => {
                    let (exe, args) = parts.split_first().unwrap();
                    let mut cmd = std::process::Command::new(exe);
                    for arg in args {
                        if arg == "%p" {
                            // -- marker prevents argument injection if file_path starts with '-'
                            cmd.arg("--");
                            cmd.arg(&file_path);
                        } else {
                            cmd.arg(arg.replace("%p", &file_path));
                        }
                    }
                    cmd.stdout(std::process::Stdio::null());
                    cmd.stderr(std::process::Stdio::null());
                    cmd.spawn().map_err(|e| e.to_string())?;
                    Ok(None)
                }
                _ => {
                    open::that(path).map_err(|e| e.to_string())?;
                    if sioyek_not_found_warning {
                        Ok(Some("⚠️ Sioyek이 감지되지 않아 기본 뷰어(Preview)로 열었습니다. (brew install --cask sioyek 필요)".to_string()))
                    } else if okular_not_found_warning {
                        Ok(Some("⚠️ Okular가 감지되지 않아 기본 뷰어(Preview)로 열었습니다. (brew install --cask okular 필요)".to_string()))
                    } else {
                        Ok(None)
                    }
                }
            }
        })
        .await;

        let action = match result {
            Ok(Ok(Some(sync_msg))) => AppAction::OpenExternalViewerResult {
                success: true,
                message: sync_msg,
            },
            Ok(Ok(None)) => AppAction::OpenExternalViewerResult {
                success: true,
                message: "PDF 열기 완료".to_string(),
            },
            Ok(Err(m)) => AppAction::OpenExternalViewerResult {
                success: false,
                message: format!("PDF 열기 실패: {}", m),
            },
            Err(e) => AppAction::OpenExternalViewerResult {
                success: false,
                message: format!("PDF 열기 실패: 태스크 오류: {}", e),
            },
        };
        let _ = tx.send(action).await;
    });
}

fn handle_open_external_result(state: &mut AppState, message: &str) {
    state.finish_processing(message);
    state.reload_documents();
    state.load_detail();
    state.dirty = true;
}

fn handle_install_sioyek(state: &mut AppState, doc_id: i64) {
    state.start_processing("Sioyek 설치 중...");
    let tx = state.action_tx.clone();

    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || -> std::result::Result<(), String> {
            #[cfg(target_os = "macos")]
            {
                let brew_path = if std::path::Path::new("/opt/homebrew/bin/brew").exists() {
                    "/opt/homebrew/bin/brew"
                } else if std::path::Path::new("/usr/local/bin/brew").exists() {
                    "/usr/local/bin/brew"
                } else {
                    "brew"
                };

                let status = std::process::Command::new(brew_path)
                    .arg("install")
                    .arg("--cask")
                    .arg("sioyek")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map_err(|e| format!("brew 명령 실행 실패: {}", e))?;
                if !status.success() {
                    return Err(format!("brew install 실패 (코드: {})", status));
                }

                // macOS Gatekeeper 격리 차단 해제 (Exit Code 126 방지)
                let _ = std::process::Command::new("xattr")
                    .arg("-r")
                    .arg("-d")
                    .arg("com.apple.quarantine")
                    .arg("/opt/homebrew/Caskroom/sioyek")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
            }

            #[cfg(target_os = "windows")]
            {
                let status = std::process::Command::new("winget")
                    .arg("install")
                    .arg("-e")
                    .arg("--id")
                    .arg("ahrm.sioyek")
                    .arg("--silent")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map_err(|e| format!("winget 명령 실행 실패: {}", e))?;
                if !status.success() {
                    return Err(format!("winget install 실패 (코드: {})", status));
                }
            }

            #[cfg(not(any(target_os = "macos", target_os = "windows")))] // Linux
            {
                let mut has_snap = false;
                if let Ok(status) = std::process::Command::new("snap")
                    .arg("--version")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                {
                    has_snap = status.success();
                }

                let status = if has_snap {
                    std::process::Command::new("sudo")
                        .arg("snap")
                        .arg("install")
                        .arg("sioyek")
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                } else {
                    std::process::Command::new("sudo")
                        .arg("apt-get")
                        .arg("install")
                        .arg("-y")
                        .arg("sioyek")
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                };

                let status = status.map_err(|e| format!("설치 프로세스 실행 실패: {}", e))?;
                if !status.success() {
                    return Err(format!("패키지 매니저 실행 실패 (코드: {})", status));
                }
            }

            Ok(())
        })
        .await;

        let action = match result {
            Ok(Ok(())) => AppAction::SioyekInstallResult {
                success: true,
                message: "Sioyek 설치 성공! PDF를 엽니다.".to_string(),
                doc_id,
            },
            Ok(Err(m)) => AppAction::SioyekInstallResult {
                success: false,
                message: format!("Sioyek 설치 실패: {}", m),
                doc_id,
            },
            Err(e) => AppAction::SioyekInstallResult {
                success: false,
                message: format!("Sioyek 설치 중 태스크 에러: {}", e),
                doc_id,
            },
        };
        let _ = tx.send(action).await;
    });
}

fn handle_sioyek_install_result(state: &mut AppState, success: bool, message: &str, doc_id: i64) {
    state.finish_processing(message);
    if success {
        state.set_status("Sioyek 설치 완료! PDF를 실행합니다.");
        let _ = state.action_tx.try_send(AppAction::OpenExternalViewer { doc_id });
    } else {
        state.set_status(&format!("설치 오류: {}", message));
    }
    state.dirty = true;
}

fn handle_sioyek_install_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            if let Some(doc_id) = state.sioyek_install_doc_id {
                state.show_sioyek_install_dialog = false;
                let _ = state.action_tx.try_send(AppAction::InstallSioyek { doc_id });
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            state.show_sioyek_install_dialog = false;
            let doc_id = state.sioyek_install_doc_id.take();
            state.set_status("Sioyek 설치 취소됨. 기본 뷰어로 엽니다.");
            
            // "아니오"를 누른 경우, Preview(시스템 기본)로 즉시 대체 실행해 줍니다.
            if let Some(id) = doc_id {
                // Sioyek 우회를 위해 is_sioyek 검증 단계를 무시하고 Preview/기본 연결앱으로 바로 열도록 direct trigger 처리
                let db = state.db.clone();
                let tx = state.action_tx.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || -> std::result::Result<(), String> {
                        let file_path = {
                            let conn = db.lock().map_err(|e| e.to_string())?;
                            crate::db::documents::get_by_id(&conn, id)
                                .map_err(|e| e.to_string())?
                                .and_then(|d| d.file_path)
                                .ok_or_else(|| "파일 경로가 없습니다".to_string())?
                        };
                        let path = std::path::Path::new(&file_path);
                        open::that(path).map_err(|e| e.to_string())?;
                        Ok(())
                    }).await;

                    let action = match result {
                        Ok(Ok(())) => AppAction::OpenExternalViewerResult {
                            success: true,
                            message: "기본 뷰어로 PDF 열기 완료".to_string(),
                        },
                        Ok(Err(m)) => AppAction::OpenExternalViewerResult {
                            success: false,
                            message: format!("기본 뷰어 열기 실패: {}", m),
                        },
                        Err(e) => AppAction::OpenExternalViewerResult {
                            success: false,
                            message: format!("기본 뷰어 열기 오류: {}", e),
                        },
                    };
                    let _ = tx.send(action).await;
                });
            }
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_install_okular(state: &mut AppState, doc_id: i64) {
    state.start_processing("Okular 설치 중...");
    let tx = state.action_tx.clone();

    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || -> std::result::Result<(), String> {
            #[cfg(target_os = "macos")]
            {
                // macOS용 공식 brew cask okular가 지원되지 않으므로 브라우저 다운로드 페이지를 열어줍니다.
                let status = std::process::Command::new("open")
                    .arg("https://okular.kde.org/download/")
                    .status()
                    .map_err(|e| format!("open 명령 실행 실패: {}", e))?;
                if !status.success() {
                    return Err(format!("브라우저 다운로드 페이지 열기 실패 (코드: {})", status));
                }
            }

            #[cfg(target_os = "windows")]
            {
                let status = std::process::Command::new("winget")
                    .arg("install")
                    .arg("-e")
                    .arg("--id")
                    .arg("KDE.Okular")
                    .arg("--silent")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map_err(|e| format!("winget 명령 실행 실패: {}", e))?;
                if !status.success() {
                    return Err(format!("winget install 실패 (코드: {})", status));
                }
            }

            #[cfg(not(any(target_os = "macos", target_os = "windows")))] // Linux
            {
                let mut has_snap = false;
                if let Ok(status) = std::process::Command::new("snap")
                    .arg("--version")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                {
                    has_snap = status.success();
                }

                let status = if has_snap {
                    std::process::Command::new("sudo")
                        .arg("snap")
                        .arg("install")
                        .arg("okular")
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                } else {
                    std::process::Command::new("sudo")
                        .arg("apt-get")
                        .arg("install")
                        .arg("-y")
                        .arg("okular")
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                };

                let status = status.map_err(|e| format!("설치 프로세스 실행 실패: {}", e))?;
                if !status.success() {
                    return Err(format!("패키지 매니저 실행 실패 (코드: {})", status));
                }
            }

            Ok(())
        })
        .await;

        let action = match result {
            Ok(Ok(())) => {
                #[cfg(target_os = "macos")]
                let msg = "Okular 다운로드 페이지를 열었습니다. 설치 후 다시 시도해 주세요.".to_string();
                #[cfg(not(target_os = "macos"))]
                let msg = "Okular 설치 성공! PDF를 엽니다.".to_string();

                AppAction::OkularInstallResult {
                    success: true,
                    message: msg,
                    doc_id,
                }
            }
            Ok(Err(m)) => AppAction::OkularInstallResult {
                success: false,
                message: format!("Okular 설치 실패: {}", m),
                doc_id,
            },
            Err(e) => AppAction::OkularInstallResult {
                success: false,
                message: format!("Okular 설치 중 태스크 에러: {}", e),
                doc_id,
            },
        };
        let _ = tx.send(action).await;
    });
}

fn handle_okular_install_result(state: &mut AppState, success: bool, message: &str, _doc_id: i64) {
    state.finish_processing(message);
    if success {
        #[cfg(target_os = "macos")]
        {
            state.set_status("다운로드 완료 후, 앱 권한 승인 후 문서를 다시 열어주세요.");
        }
        #[cfg(not(target_os = "macos"))]
        {
            state.set_status("Okular 설치 완료! PDF를 실행합니다.");
            let _ = state.action_tx.try_send(AppAction::OpenExternalViewer { doc_id });
        }
    } else {
        state.set_status(&format!("설치 오류: {}", message));
    }
    state.dirty = true;
}

fn handle_okular_install_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            if let Some(doc_id) = state.okular_install_doc_id {
                state.show_okular_install_dialog = false;
                let _ = state.action_tx.try_send(AppAction::InstallOkular { doc_id });
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            state.show_okular_install_dialog = false;
            let doc_id = state.okular_install_doc_id.take();
            state.set_status("Okular 설치 취소됨. 기본 뷰어로 엽니다.");
            
            // "아니오"를 누른 경우, Preview(시스템 기본)로 즉시 대체 실행해 줍니다.
            if let Some(id) = doc_id {
                let db = state.db.clone();
                let tx = state.action_tx.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || -> std::result::Result<(), String> {
                        let file_path = {
                            let conn = db.lock().map_err(|e| e.to_string())?;
                            crate::db::documents::get_by_id(&conn, id)
                                .map_err(|e| e.to_string())?
                                .and_then(|d| d.file_path)
                                .ok_or_else(|| "파일 경로가 없습니다".to_string())?
                        };
                        let path = std::path::Path::new(&file_path);
                        open::that(path).map_err(|e| e.to_string())?;
                        Ok(())
                    }).await;

                    let action = match result {
                        Ok(Ok(())) => AppAction::OpenExternalViewerResult {
                            success: true,
                            message: "기본 뷰어로 PDF 열기 완료".to_string(),
                        },
                        Ok(Err(m)) => AppAction::OpenExternalViewerResult {
                            success: false,
                            message: format!("기본 뷰어 열기 실패: {}", m),
                        },
                        Err(e) => AppAction::OpenExternalViewerResult {
                            success: false,
                            message: format!("기본 뷰어 열기 오류: {}", e),
                        },
                    };
                    let _ = tx.send(action).await;
                });
            }
            state.dirty = true;
        }
        _ => {}
    }
    false
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
                && source_id != target_id
            {
                let db = state.db.clone();
                if let Ok(conn) = db.lock() {
                    if let Err(e) = entry::add_manual_citation(&conn, source_id, target_id) {
                        tracing::error!("Failed to add manual citation {source_id}→{target_id}: {e}");
                    }
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
            let _ = state
                .action_tx
                .try_send(AppAction::StartBibtexImport { doc_id, path });
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
                if let Err(e) = entry::add_bibtex_citation(&conn, doc_id, target_id) {
                    tracing::error!("Failed to add BibTeX citation for doc {doc_id}→{target_id}: {e}");
                }
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
        let result = tokio::task::spawn_blocking(
            move || -> std::result::Result<(CitationGraph, bool), String> {
                let conn = db.lock().map_err(|e| e.to_string())?;
                let cache_key = cache::build_cache_key(&doc_ids);

                let cache_hit =
                    !cache::should_regenerate(&conn, &cache_key, &doc_ids).unwrap_or(true);

                let graph = CitationGraph::build(&conn, &doc_ids).map_err(|e| e.to_string())?;

                if !cache_hit {
                    let edge_version = cache::compute_edge_version(&conn, &doc_ids).unwrap_or(0);
                    let node_count = graph.node_count();
                    let render_mode = graph::RenderMode::for_node_count(node_count);
                    let graph_data = format!(
                        "{{\"nodes\":{},\"edges\":{}}}",
                        node_count,
                        graph.inner.edge_count()
                    );
                    let _ = cache::store_cache(
                        &conn,
                        &cache_key,
                        &graph_data,
                        edge_version,
                        doc_ids.len() as i64,
                        &render_mode,
                    );
                }

                Ok((graph, cache_hit))
            },
        )
        .await;

        match result {
            Ok(Ok((g, cache_hit))) => {
                let cache_key = cache::build_cache_key(&doc_ids_clone);
                let gs = GraphState::new(g, cache_hit);
                let _ = tx
                    .send(AppAction::CitationGraphReady {
                        graph_state: Box::new(gs),
                        cache_key,
                        cache_hit,
                    })
                    .await;
            }
            Ok(Err(reason)) => {
                let _ = tx
                    .send(AppAction::OperationFailed(format!(
                        "그래프 생성 실패: {}",
                        reason
                    )))
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppAction::OperationFailed(format!("태스크 실패: {}", e)))
                    .await;
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
            let _ = state.action_tx.try_send(AppAction::NavigateGraph {
                direction: GraphDirection::Down,
            });
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let _ = state.action_tx.try_send(AppAction::NavigateGraph {
                direction: GraphDirection::Up,
            });
        }
        KeyCode::Char('h') | KeyCode::Left => {
            let _ = state.action_tx.try_send(AppAction::NavigateGraph {
                direction: GraphDirection::Left,
            });
        }
        KeyCode::Char('l') | KeyCode::Right => {
            let _ = state.action_tx.try_send(AppAction::NavigateGraph {
                direction: GraphDirection::Right,
            });
        }
        KeyCode::Tab => {
            let _ = state.action_tx.try_send(AppAction::ToggleGraphRenderMode);
        }
        KeyCode::Char('G') => {
            if let Some(ref gs) = state.graph_state {
                let doc_ids = gs.doc_ids.clone();
                let _ = state
                    .action_tx
                    .try_send(AppAction::GenerateCitationGraph { doc_ids });
            }
        }
        KeyCode::Enter => {
            if let Some(ref gs) = state.graph_state
                && let Some(node_idx) = gs.focused_node
            {
                let _ = state
                    .action_tx
                    .try_send(AppAction::SelectGraphNode { node_idx });
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

fn handle_command_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.command_mode = false;
            state.command_input.clear();
            state.set_status("취소됨");
            state.dirty = true;
        }
        KeyCode::Enter => {
            let input = state.command_input.trim().to_string();
            state.command_mode = false;
            state.command_input.clear();
            if input.is_empty() {
                state.set_status("준비됨");
            } else if let Some((cmd, arg)) = split_command(&input) {
                match cmd {
                    "backup" => {
                        if arg.is_empty() {
                            state.set_status("사용법: :backup <파일경로>");
                        } else {
                            let _ = state.action_tx.try_send(AppAction::Backup {
                                path: arg.to_string(),
                            });
                        }
                    }
                    "restore" => {
                        if arg.is_empty() {
                            state.set_status("사용법: :restore <파일경로>");
                        } else {
                            let _ = state.action_tx.try_send(AppAction::Restore {
                                path: arg.to_string(),
                            });
                        }
                    }
                    "import-db" => {
                        if arg.is_empty() {
                            state.set_status("사용법: :import-db <백업파일경로>");
                        } else {
                            let _ = state.action_tx.try_send(AppAction::ImportDb {
                                path: arg.to_string(),
                            });
                        }
                    }
                    "tag-color" => {
                        let parts: Vec<&str> = arg.split_whitespace().collect();
                        if parts.is_empty() {
                            state.set_status(
                                "사용법: :tag-color <태그> <색상>  (색상 생략 시 색상 삭제)",
                            );
                        } else {
                            let tag = parts[0].to_string();
                            let color = parts.get(1).map(|s| s.to_string());
                            let _ = state
                                .action_tx
                                .try_send(AppAction::SetTagColor { tag, color });
                        }
                    }
                    "import-classification" => {
                        if arg.is_empty() {
                            state.set_status("사용법: :import-classification <CSV 파일 경로>");
                        } else {
                            let _ = state.action_tx.try_send(AppAction::ImportClassification {
                                path: arg.to_string(),
                            });
                        }
                    }
                    "note" => {
                        handle_note_editor(state);
                    }
                    _ => {
                        state.set_status(&format!("알 수 없는 명령: :{}", cmd));
                    }
                }
            }
            state.dirty = true;
        }
        KeyCode::Backspace => {
            state.command_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.command_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn split_command(input: &str) -> Option<(&str, &str)> {
    let input = input.strip_prefix(':').unwrap_or(input);
    let mut parts = input.splitn(2, char::is_whitespace);
    let cmd = parts.next()?.trim();
    let arg = parts.next().unwrap_or("").trim();
    if cmd.is_empty() {
        None
    } else {
        Some((cmd, arg))
    }
}

fn handle_backup(state: &mut AppState, path: String) {
    let dest = PathBuf::from(&path);
    state.start_processing("데이터베이스 백업 중...");

    let db_conn = state.db.clone();
    let action_tx = state.action_tx.clone();

    std::thread::spawn(move || {
        let result = {
            if let Ok(conn) = db_conn.lock() {
                crate::db::backup::backup_to_path(&conn, &dest)
            } else {
                Err(anyhow::anyhow!("DB 락 획득 실패"))
            }
        };

        let success = result.is_ok();
        let msg = match result {
            Ok(_) => format!("백업 완료: {}", dest.display()),
            Err(e) => format!("백업 실패: {}", e),
        };

        let _ = action_tx.try_send(AppAction::ProcessFinished {
            success,
            message: msg,
        });
    });
}

fn handle_restore(state: &mut AppState, path: String) {
    let src = PathBuf::from(&path);
    let dest = state.config.db_path.clone();

    // Close the active DB connection so fs::copy (inside restore_from_path)
    // does not fail with a "file in use" error on Windows, and to avoid
    // silently writing through a stale fd on Linux/macOS.
    if let Ok(mut db_guard) = state.db.lock() {
        let _old_conn = std::mem::replace(
            &mut *db_guard,
            rusqlite::Connection::open_in_memory()
                .expect("in-memory SQLite open should never fail"),
        );
        // db_guard now holds a harmless in-memory connection; the original
        // connection to `dest` is dropped. The user *must* restart after restore.
    }

    let result = crate::db::backup::restore_from_path(&src, &dest);
    match result {
        Ok(_) => state.set_status(&format!("복원 완료: {} (재시작 필요)", path)),
        Err(e) => state.set_status(&format!("복원 실패: {}", e)),
    }
    state.dirty = true;
}

fn handle_import_db(state: &mut AppState, path: String) {
    let src = PathBuf::from(&path);
    if !src.exists() {
        state.set_status(&format!("오류: 파일을 찾을 수 없습니다: {}", path));
        return;
    }
    state.start_processing("데이터베이스 불러오기 (Import) 중...");
    
    let db_conn = state.db.clone();
    let action_tx = state.action_tx.clone();
    
    std::thread::spawn(move || {
        let result = {
            if let Ok(mut conn) = db_conn.lock() {
                crate::db::backup::import_db_from_path(&mut *conn, &src)
            } else {
                Err(anyhow::anyhow!("DB 락 획득 실패"))
            }
        };
        
        let success = result.is_ok();
        let msg = match result {
            Ok((s, skip)) => {
                format!("불러오기 완료: {}건 병합, {}건 중복 스킵", s, skip)
            }
            Err(e) => {
                format!("불러오기 실패: {}", e)
            }
        };
        
        let _ = action_tx.try_send(AppAction::ProcessFinished {
            success,
            message: msg,
        });
    });
}

fn handle_toggle_favorite_filter(state: &mut AppState) {
    state.favorite_filter = !state.favorite_filter;
    state.list_cursor = 0;
    state.reload_documents();
    if state.favorite_filter {
        state.set_status("즐겨찾기 필터 켜짐 (별점 5점 문헌만 표시)");
    } else {
        state.set_status("즐겨찾기 필터 꺼짐 (전체 문헌 표시)");
    }
    state.dirty = true;
}

fn handle_set_tag_color(state: &mut AppState, tag: &str, color: Option<String>) {
    let err = {
        state.db.lock().ok().and_then(|conn| {
            documents::set_tag_color(&conn, tag, color.as_deref()).err()
        })
    };
    if let Some(e) = err {
        tracing::error!("set_tag_color 실패 (tag={}): {}", tag, e);
        state.set_status(&format!("태그 색상 설정 실패: {}", e));
    } else {
        state.reload_tags();
        match color {
            Some(c) => state.set_status(&format!("태그 색상 설정: {} → {}", tag, c)),
            None => state.set_status(&format!("태그 색상 삭제: {}", tag)),
        }
    }
    state.dirty = true;
}

fn handle_toggle_queue_view(state: &mut AppState) {
    state.queue_view = !state.queue_view;
    state.list_cursor = 0;
    if state.queue_view {
        state.reload_queue();
        state.set_status(&format!("읽기 큐 보기 ({}건)", state.queue.len()));
    } else {
        state.set_status("읽기 큐 보기 종료");
        state.reload_documents();
    }
    state.dirty = true;
}

fn handle_add_to_queue(state: &mut AppState, doc_id: i64) {
    let err = {
        state.db.lock().ok().and_then(|conn| {
            documents::add_to_queue(&conn, doc_id).err()
        })
    };
    if let Some(e) = err {
        tracing::error!("add_to_queue 실패 (doc_id={}): {}", doc_id, e);
        state.set_status(&format!("읽기 큐 추가 실패: {}", e));
    } else {
        state.set_status("읽기 큐에 추가됨");
    }
    state.dirty = true;
}

fn handle_remove_from_queue(state: &mut AppState, doc_id: i64) {
    let err = {
        state.db.lock().ok().and_then(|conn| {
            documents::remove_from_queue(&conn, doc_id).err()
        })
    };
    if let Some(e) = err {
        tracing::error!("remove_from_queue 실패 (doc_id={}): {}", doc_id, e);
        state.set_status(&format!("읽기 큐 제거 실패: {}", e));
    } else {
        if state.queue_view {
            state.reload_queue();
            if state.list_cursor >= state.queue.len() && !state.queue.is_empty() {
                state.list_cursor = state.queue.len() - 1;
            }
        }
        state.set_status("읽기 큐에서 제거됨");
    }
    state.dirty = true;
}

fn handle_reorder_queue(state: &mut AppState, doc_id: i64, new_position: usize) {
    let err = {
        state.db.lock().ok().and_then(|conn| {
            documents::reorder_queue(&conn, doc_id, new_position).err()
        })
    };
    if let Some(e) = err {
        tracing::error!("reorder_queue 실패 (doc_id={}, pos={}): {}", doc_id, new_position, e);
        state.set_status(&format!("읽기 큐 순서 변경 실패: {}", e));
    } else {
        state.reload_queue();
    }
    state.dirty = true;
}

fn handle_update_reading_progress(state: &mut AppState, doc_id: i64, progress: i64) {
    let err = {
        state.db.lock().ok().and_then(|conn| {
            documents::update_reading_progress(&conn, doc_id, progress).err()
        })
    };
    if let Some(e) = err {
        tracing::error!("update_reading_progress 실패 (doc_id={}, progress={}): {}", doc_id, progress, e);
        state.set_status(&format!("읽기 진행률 업데이트 실패: {}", e));
    }
    if let Some(doc) = state.documents.iter_mut().find(|d| d.id == Some(doc_id)) {
        doc.reading_progress = Some(progress);
    }
    if let Some(doc) = state.detail_doc.as_mut()
        && doc.id == Some(doc_id)
    {
        doc.reading_progress = Some(progress);
    }
    state.set_status(&format!("읽기 진행률: {}%", progress));
    state.dirty = true;
}

fn handle_queue_view_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('Y') | KeyCode::Char('q') => {
            let _ = state.action_tx.try_send(AppAction::ToggleQueueView);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if state.list_cursor + 1 < state.queue.len() {
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
        KeyCode::Char('R') => {
            if let Some(doc) = state.queue.get(state.list_cursor)
                && let Some(id) = doc.id
            {
                let _ = state
                    .action_tx
                    .try_send(AppAction::RemoveFromQueue { doc_id: id });
            }
        }
        KeyCode::Char('J') => {
            if let Some(doc) = state.queue.get(state.list_cursor)
                && let Some(id) = doc.id
                && state.list_cursor + 1 < state.queue.len()
            {
                let _ = state.action_tx.try_send(AppAction::ReorderQueue {
                    doc_id: id,
                    new_position: state.list_cursor + 1,
                });
                state.list_cursor += 1;
            }
        }
        KeyCode::Char('K') => {
            if let Some(doc) = state.queue.get(state.list_cursor)
                && let Some(id) = doc.id
                && state.list_cursor > 0
            {
                let _ = state.action_tx.try_send(AppAction::ReorderQueue {
                    doc_id: id,
                    new_position: state.list_cursor - 1,
                });
                state.list_cursor -= 1;
            }
        }
        KeyCode::Enter => {
            if let Some(doc) = state.queue.get(state.list_cursor).cloned() {
                state.queue_view = false;
                state.show_detail = true;
                state.detail_doc = Some(doc);
                state.active_panel = PanelFocus::Detail;
                state.dirty = true;
            }
        }
        KeyCode::Char('?') => {
            state.show_help = true;
            state.help_page = 0;
            state.dirty = true;
        }
        _ => {}
    }
    false
}

fn handle_import_classification(state: &mut AppState, path: &str) {
    let result = {
        if let Ok(conn) = state.db.lock() {
            crate::classification::csv_import::import_classification_csv(
                &conn,
                std::path::Path::new(path),
            )
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(scheme_id) => {
            state.reload_udc_tree();
            state.set_status(&format!(
                "분류 체계 가져오기 완료 (scheme_id={})",
                scheme_id
            ));
        }
        Err(e) => state.set_status(&format!("가져오기 실패: {}", e)),
    }
}

fn handle_note_editor(state: &mut AppState) {
    let doc_id = if state.show_detail {
        state.detail_doc.as_ref().and_then(|d| d.id)
    } else if state.active_panel == PanelFocus::Right {
        state.documents.get(state.list_cursor).and_then(|d| d.id)
    } else {
        None
    };

    let Some(doc_id) = doc_id else {
        state.set_status("노트를 작성할 문헌을 선택하세요");
        return;
    };

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let mut temp_path = std::env::temp_dir();
    temp_path.push(format!("libran_note_{}.md", doc_id));

    if let Err(e) = std::fs::write(&temp_path, "") {
        state.set_status(&format!("임시 파일 생성 실패: {}", e));
        return;
    }

    let result = std::process::Command::new(&editor).arg(&temp_path).status();

    match result {
        Ok(status) if status.success() => {
            let content = std::fs::read_to_string(&temp_path).unwrap_or_default();
            let _ = std::fs::remove_file(&temp_path);
            let trimmed = content.trim_end();
            if trimmed.is_empty() {
                state.set_status("노트가 비어 있어 저장하지 않음");
            } else {
                let saved = state.db.lock().ok().and_then(|conn| {
                    if let Err(e) = crate::db::notes::create(&conn, doc_id, trimmed, "general") {
                        tracing::error!("note_editor: notes::create failed: {} (doc_id={})", e, doc_id);
                        return None;
                    }
                    Some(crate::db::notes::list(&conn, doc_id).unwrap_or_default())
                });
                match saved {
                    Some(notes) => {
                        state.current_notes = notes;
                        state.set_status("노트 저장됨 ($EDITOR)");
                    }
                    None => state.set_status("노트 저장 실패"),
                }
            }
        }
        Ok(_) => state.set_status("에디터가 정상 종료되지 않음"),
        Err(e) => state.set_status(&format!("에디터 실행 실패: {}", e)),
    }
    state.dirty = true;
}

// ── Widget Panel Key Handler ───────────────────────────────────────────────────

fn handle_widget_panel_key(state: &mut AppState, key: KeyEvent) -> bool {
    use crossterm::event::KeyCode;

    if matches!(key.code, KeyCode::Esc | KeyCode::Char('w')) {
        state.show_widget_panel = false;
        state.dirty = true;
        return false;
    }

    match key.code {
        KeyCode::Tab => {
            state.widget_registry.next_tab();
            state.dirty = true;
            return false;
        }
        KeyCode::BackTab => {
            state.widget_registry.prev_tab();
            state.dirty = true;
            return false;
        }
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let idx = (c as usize) - ('1' as usize);
            state.widget_registry.switch_tab(idx);
            state.dirty = true;
            return false;
        }
        KeyCode::Char('y') => {
            // 보안 경고 수락: 현재 위젯의 스크립트 실행을 허용
            if let Some(widget) = state.widget_registry.active_widget() {
                if matches!(widget, crate::widget::WidgetInstance::Script(_)) {
                    let id = widget.id().to_string();
                    state.widget_registry.acknowledge_security_warning(&id);
                    spawn_widget_fetch(state, &id);
                    state.dirty = true;
                }
            }
            return false;
        }
        KeyCode::Char('r') => {
            // 수동 새로고침: 현재 위젯의 fetch를 spawn
            if let Some(widget) = state.widget_registry.active_widget() {
                let id = widget.id().to_string();
                spawn_widget_fetch(state, &id);
            }
            state.dirty = true;
            return false;
        }
        _ => {}
    }

    false
}

// ── Widget Async Fetch Helpers ────────────────────────────────────────────────

/// 특정 위젯의 데이터를 비동기로 fetch합니다 (수동 새로고침용).
fn spawn_widget_fetch(state: &mut AppState, widget_id: &str) {
    use crate::widget::WidgetInstance;

    let action_tx = state.action_tx.clone();

    // Check security approval before executing script widgets.
    if !state.widget_registry.is_security_approved(widget_id) {
        // If the widget is a Script widget with an unacknowledged security warning,
        // show the warning instead of executing.
        state.widget_registry.set_content(
            widget_id,
            crate::widget::WidgetContent::error("보안 경고: 이 위젯은 외부 스크립트를 실행합니다. 실행하려면 위젯 패널에서 'y'를 누르세요."),
        );
        return;
    }

    state
        .widget_registry
        .set_content(widget_id, crate::widget::WidgetContent::loading());

    if let Some(instance) = state.widget_registry.widgets.iter_mut().find(|w| w.id() == widget_id) {
        match instance {
            WidgetInstance::Api(runner) => {
                spawn_api_fetch(&action_tx, runner);
            }
            WidgetInstance::Script(runner) => {
                spawn_script_fetch(&action_tx, runner);
            }
        }
    }
}

fn spawn_api_fetch(
    action_tx: &tokio::sync::mpsc::Sender<AppAction>,
    runner: &crate::widget::api_runner::ApiWidgetRunner,
) {
    let id = runner.id().to_string();
    let temp = crate::widget::api_runner::ApiWidgetRunner::new(
        runner.meta.clone(),
        runner.api_cfg.clone(),
        runner.display_cfg.clone(),
        runner.sandbox.clone(),
    );
    let tx = action_tx.clone();
    tokio::spawn(async move {
        match temp.fetch().await {
            Ok(content) => {
                let _ = tx.try_send(AppAction::WidgetDataReady {
                    widget_id: id,
                    content: Box::new(content),
                });
            }
            Err(e) => {
                let _ = tx.try_send(AppAction::WidgetError {
                    widget_id: id,
                    message: e.to_string(),
                });
            }
        }
    });
}

fn spawn_script_fetch(
    action_tx: &tokio::sync::mpsc::Sender<AppAction>,
    runner: &crate::widget::script_runner::ScriptWidgetRunner,
) {
    let id = runner.id().to_string();
    let temp = crate::widget::script_runner::ScriptWidgetRunner::new(
        runner.meta.clone(),
        runner.script_cfg.clone(),
        runner.widget_dir.clone(),
        runner.permissions.clone(),
    );
    let tx = action_tx.clone();
    tokio::spawn(async move {
        match temp.run().await {
            Ok(content) => {
                let _ = tx.try_send(AppAction::WidgetDataReady {
                    widget_id: id,
                    content: Box::new(content),
                });
            }
            Err(e) => {
                let _ = tx.try_send(AppAction::WidgetError {
                    widget_id: id,
                    message: e.to_string(),
                });
            }
        }
    });
}

// ── Widget Tick Handler ────────────────────────────────────────────────────────

fn handle_widget_tick(state: &mut AppState) {
    use crate::widget::WidgetInstance;

    let action_tx = state.action_tx.clone();

    let approved_ids: std::collections::HashSet<String> = state
        .widget_registry
        .widgets
        .iter()
        .filter(|w| state.widget_registry.is_security_approved(w.id()))
        .map(|w| w.id().to_string())
        .collect();

    for (idx, instance) in state.widget_registry.widgets.iter_mut().enumerate() {
        match instance {
            WidgetInstance::Api(runner) => {
                if runner.needs_refresh() {
                    if idx < state.widget_registry.contents.len() {
                        state.widget_registry.contents[idx] =
                            crate::widget::WidgetContent::loading();
                    }
                    spawn_api_fetch(&action_tx, runner);
                }
            }
            WidgetInstance::Script(runner) => {
                if runner.needs_refresh() && approved_ids.contains(runner.id()) {
                    if idx < state.widget_registry.contents.len() {
                        state.widget_registry.contents[idx] =
                            crate::widget::WidgetContent::loading();
                    }
                    spawn_script_fetch(&action_tx, runner);
                }
            }
        }
    }
    state.dirty = true;
}
