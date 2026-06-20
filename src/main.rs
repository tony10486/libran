use anyhow::Result;
use crossterm::event::{EventStream, KeyEventKind, MouseButton, MouseEventKind};
use futures::StreamExt;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use libran::app::{dispatcher, AppAction, AppState};
use libran::config::AppConfig;
use libran::db;
use libran::terminal;
use libran::ui;

static LAST_HOVER_ROW: AtomicU16 = AtomicU16::new(u16::MAX);

fn init_logging() {
    use tracing_subscriber::fmt;

    let log_path = directories::BaseDirs::new()
        .map(|d| d.home_dir().join(".libran/libran.log"))
        .unwrap_or_else(|| std::path::PathBuf::from("libran.log"));

    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(f) => f,
        Err(_) => return,
    };

    fmt()
        .with_writer(std::sync::Mutex::new(file))
        .with_ansi(false)
        .with_target(false)
        .with_max_level(tracing::Level::DEBUG)
        .try_init()
        .ok();
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    init_logging();
    info!("Libran 시작");

    let config = AppConfig::load();
    info!("DB 경로: {:?}", config.db_path);

    let db_conn = db::open_database(&config.db_path)?;
    info!("DB 연결 완료");

    let (action_tx, mut action_rx) = mpsc::channel::<AppAction>(256);

    let mut state = AppState::new(db_conn, config, action_tx.clone());
    state.init_classification();
    state.reload_projects();
    state.reload_documents();
    info!("초기화 완료: {} 문헌", state.document_count);

    let mut terminal = terminal::setup_terminal()?;
    info!("터미널 설정 완료");

    let mut event_stream = EventStream::new();

    loop {
        if state.dirty {
            terminal.draw(|frame| ui::render(frame, &state))?;
            state.clear_dirty();
        }

        tokio::select! {
            maybe_event = event_stream.next() => {
                if let Some(result) = maybe_event {
                    match result {
                        Ok(event) => {
                            debug!("터미널 이벤트: {:?}", event);
                            if let Some(act) = convert_terminal_event(&event) {
                                let should_quit = dispatcher::handle_action(&mut state, act);
                                if should_quit {
                                    info!("종료 요청");
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("이벤트 읽기 오류: {}", e);
                        }
                    }
                }
            }
            maybe_action = action_rx.recv() => {
                if let Some(act) = maybe_action {
                    debug!("액션: {:?}", act);
                    let should_quit = dispatcher::handle_action(&mut state, act);
                    if should_quit {
                        info!("종료 요청");
                        break;
                    }
                }
            }
        }
    }

    terminal::restore_terminal(&mut terminal)?;
    info!("Libran 종료");

    Ok(())
}

fn convert_terminal_event(event: &crossterm::event::Event) -> Option<AppAction> {
    match event {
        crossterm::event::Event::Key(key) => {
            if key.kind == KeyEventKind::Release {
                None
            } else {
                Some(AppAction::KeyPressed(*key))
            }
        }
        crossterm::event::Event::Paste(text) => {
            info!("Paste 이벤트 수신: {:?}", text);
            let path = libran::terminal::drag_drop::parse_dragged_path(text);
            if path.is_some() {
                info!("경로 파싱 성공: {:?}", path);
            } else {
                warn!("경로 파싱 실패: {:?}", text);
            }
            path.map(AppAction::DragDetected)
        }
        crossterm::event::Event::Mouse(mouse) => {
            match mouse.kind {
                MouseEventKind::Moved => {
                    if mouse.row == LAST_HOVER_ROW.swap(mouse.row, Ordering::Relaxed) {
                        None
                    } else {
                        Some(AppAction::MouseHover {
                            column: mouse.column,
                            row: mouse.row,
                        })
                    }
                }
                MouseEventKind::Down(MouseButton::Left) => {
                    LAST_HOVER_ROW.store(mouse.row, Ordering::Relaxed);
                    Some(AppAction::MouseClick {
                        column: mouse.column,
                        row: mouse.row,
                    })
                }
                _ => {
                    debug!("마우스 이벤트: {:?}", mouse);
                    None
                }
            }
        }
        crossterm::event::Event::FocusGained => {
            debug!("포커스 획득");
            None
        }
        crossterm::event::Event::FocusLost => {
            debug!("포커스 상실");
            None
        }
        crossterm::event::Event::Resize(w, h) => {
            debug!("리사이즈: {}x{}", w, h);
            LAST_HOVER_ROW.store(u16::MAX, Ordering::Relaxed);
            Some(AppAction::TerminalResize { width: *w, height: *h })
        }
    }
}
