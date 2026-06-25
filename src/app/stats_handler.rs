use crate::app::AppState;
use crate::db::stats;

/// Toggle the stats dashboard. When turning on, compute stats synchronously.
pub fn handle_toggle_stats_dashboard(state: &mut AppState) {
    if state.show_stats {
        state.show_stats = false;
        state.library_stats = None;
        state.set_status("");
        return;
    }

    let db = state.db.clone();
    if let Ok(conn) = db.lock() {
        match stats::compute(&conn) {
            Ok(s) => {
                state.library_stats = Some(s);
                state.show_stats = true;
                state.set_status("통계 대시보드 (i로 닫기)");
            }
            Err(e) => {
                state.set_status(&format!("통계 계산 실패: {}", e));
            }
        }
    } else {
        state.set_status("DB 잠금 실패");
    }
}
