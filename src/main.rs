use anyhow::Result;
use crossterm::event::{EventStream, KeyEventKind, MouseButton, MouseEventKind};
use futures::StreamExt;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use libran::app::{AppAction, AppState, dispatcher};
use libran::config::AppConfig;
use libran::db;
use libran::terminal;
use libran::ui;

static LAST_HOVER_ROW: AtomicU16 = AtomicU16::new(u16::MAX);
static LAST_HOVER_COL: AtomicU16 = AtomicU16::new(u16::MAX);

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

    ui::theme::init_theme(ui::theme::load_theme(&config));

    // 위젯 틱 간격을 설정에서 읽어옵니다 (config은 AppState::new로 소비됨)
    let widget_tick_interval = Duration::from_secs(config.widgets.tick_interval_secs);

    let mut state = AppState::new(db_conn, config, action_tx.clone());
    state.init_classification();
    state.reload_projects();
    state.reload_series();
    state.reload_documents();
    info!("초기화 완료: {} 문헌", state.document_count);

    let mut terminal = terminal::setup_terminal()?;
    info!("터미널 설정 완료");

    let mut event_stream = EventStream::new();

    // 위젯 자동 갱신을 위한 틱 (설정 가능 간격, 기본 1초)
    let mut widget_tick = tokio::time::interval(widget_tick_interval);
    widget_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        if state.dirty {
            // terminal_size를 frame과 동기화: mouse handler가 올바른 좌표를 사용하도록
            if let Ok((w, h)) = crossterm::terminal::size() {
                state.terminal_size = (w, h);
            }
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
            _ = widget_tick.tick() => {
                let _ = action_tx.try_send(AppAction::WidgetTick);
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
        crossterm::event::Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::Moved => {
                let old_row = LAST_HOVER_ROW.swap(mouse.row, Ordering::Relaxed);
                let old_col = LAST_HOVER_COL.swap(mouse.column, Ordering::Relaxed);
                if mouse.row == old_row && mouse.column == old_col {
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
                LAST_HOVER_COL.store(mouse.column, Ordering::Relaxed);
                Some(AppAction::MouseClick {
                    column: mouse.column,
                    row: mouse.row,
                })
            }
            _ => {
                debug!("마우스 이벤트: {:?}", mouse);
                None
            }
        },
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
            LAST_HOVER_COL.store(u16::MAX, Ordering::Relaxed);
            Some(AppAction::TerminalResize {
                width: *w,
                height: *h,
            })
        }
    }
}
