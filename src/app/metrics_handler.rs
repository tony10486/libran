use crossterm::event::{KeyCode, KeyEvent};

use crate::api::metrics::{AuthorMetrics, MetricsBackend};

use super::AppState;
use super::action::AppAction;
use super::dispatcher::normalize_korean_key;

pub(crate) fn handle_metrics_overlay_key(state: &mut AppState, key: KeyEvent) -> bool {
    let key = normalize_korean_key(key);
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
            state.show_metrics_overlay = false;
            state.metrics_overlay_name.clear();
            state.dirty = true;
        }
        _ => {}
    }
    false
}

pub(crate) fn handle_api_key_input_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.api_key_input_mode = false;
            state.api_key_input.clear();
            state.set_status("준비됨");
        }
        KeyCode::Enter => {
            let key_input = state.api_key_input.clone();
            state.api_key_input_mode = false;
            state.api_key_input.clear();
            let _ = state
                .action_tx
                .try_send(AppAction::RegisterApiKey(key_input));
        }
        KeyCode::Backspace => {
            state.api_key_input.pop();
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            state.api_key_input.push(c);
            state.dirty = true;
        }
        _ => {}
    }
    false
}

pub(crate) fn handle_fetch_author_metrics(state: &mut AppState, name: String) {
    if !state.api_mode.allows_api_calls() {
        state.set_status("오프라인 모드입니다 (o 키로 전환)");
        return;
    }
    if let Some(existing) = state.author_metrics.get(&name) {
        state.show_metrics_overlay = true;
        state.metrics_overlay_name = name.clone();
        state.set_status(&format!(
            "캐시된 지표: {} (h={}, 출처: {})",
            existing.name,
            existing.h_index.unwrap_or(0),
            existing.source.display_name()
        ));
        state.dirty = true;
        return;
    }

    let backend = state.metrics_backend;
    let max_age = state.metrics_refresh_interval_days;
    let cached_metrics = {
        if let Ok(conn) = state.db.lock() {
            crate::api::metrics::get_cached_metrics(&conn, backend, &name, max_age)
                .ok()
                .flatten()
        } else {
            None
        }
    };
    if let Some(cached) = cached_metrics {
        state.author_metrics.insert(name.clone(), cached.clone());
        state.show_metrics_overlay = true;
        state.metrics_overlay_name = name.clone();
        state.set_status(&format!(
            "캐시된 지표: {} (h={}, 출처: {})",
            cached.name,
            cached.h_index.unwrap_or(0),
            cached.source.display_name()
        ));
        state.dirty = true;
        return;
    }

    let backend = state.metrics_backend;
    let api_key = state.openalex_api_key.clone();
    let tx = state.action_tx.clone();
    let name_clone = name.clone();
    let db = state.db.clone();
    let stale_metrics = if let Ok(conn) = db.lock() {
        crate::api::metrics::get_stale_cached_metrics(&conn, backend, &name_clone)
            .ok()
            .flatten()
    } else {
        None
    };
    tokio::spawn(async move {
        match crate::api::metrics::fetch_author_metrics(backend, api_key.as_deref(), &name_clone)
            .await
        {
            Ok(metrics) => {
                if let Ok(conn) = db.lock() {
                    let _ = crate::api::metrics::store_cached_metrics(
                        &conn,
                        backend,
                        &name_clone,
                        &metrics,
                    );
                }
                let _ = tx
                    .send(AppAction::AuthorMetricsFetched {
                        name: name_clone,
                        metrics: Box::new(metrics),
                    })
                    .await;
            }
            Err(e) => {
                let err_str = e.to_string();
                if (err_str.contains("429") || err_str.contains("제한"))
                    && let Some(stale) = stale_metrics
                {
                    let _ = tx
                        .send(AppAction::AuthorMetricsFetched {
                            name: name_clone,
                            metrics: Box::new(stale),
                        })
                        .await;
                    return;
                }
                let _ = tx
                    .send(AppAction::AuthorMetricsFailed {
                        name: name_clone,
                        reason: err_str,
                    })
                    .await;
            }
        }
    });
}

pub(crate) fn handle_author_metrics_fetched(
    state: &mut AppState,
    name: String,
    metrics: AuthorMetrics,
) {
    state.author_metrics.insert(name.clone(), metrics.clone());
    state.show_metrics_overlay = true;
    state.metrics_overlay_name = name.clone();
    state.set_status(&format!(
        "지표 조회 완료: {} (h={})",
        name,
        metrics.h_index.unwrap_or(0)
    ));
    state.dirty = true;
}

pub(crate) fn handle_set_metrics_backend(state: &mut AppState, backend: MetricsBackend) {
    state.metrics_backend = backend;
    if let Ok(conn) = state.db.lock() {
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO app_config (key, value, updated_at) \
             VALUES ('metrics_backend', ?1, datetime('now'))",
            rusqlite::params![backend.as_str()],
        ) {
            tracing::error!("Failed to save metrics_backend setting: {e}");
        }
    }
    state.set_status(&format!("지표 백엔드: {}", backend.display_name()));
}

pub(crate) fn handle_register_api_key(state: &mut AppState, key: String) {
    let key_trimmed = key.trim().to_string();
    if key_trimmed.is_empty() {
        state.set_status("API 키가 비어 있어 백엔드를 Semantic Scholar로 전환합니다");
        handle_set_metrics_backend(state, MetricsBackend::SemanticScholar);
        return;
    }
    state.openalex_api_key = Some(key_trimmed.clone());
    if let Ok(conn) = state.db.lock() {
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO app_config (key, value, updated_at) \
             VALUES ('openalex_api_key', ?1, datetime('now'))",
            rusqlite::params![key_trimmed],
        ) {
            tracing::error!("Failed to save openalex_api_key setting: {e}");
        }
    }
    handle_set_metrics_backend(state, MetricsBackend::OpenAlex);
    state.set_status("OpenAlex API 키 등록 완료");
}
