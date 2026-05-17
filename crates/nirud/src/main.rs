mod db;
mod ipc;
mod timer;

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, mpsc};

use niru_core::ipc::{Command, Event};
use niru_core::models::{Config, Phase, Session};

use db::Db;
use timer::Timer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    log::info!("nirud starting");

    let config = Config::default();

    let (event_tx, _) = broadcast::channel::<Event>(64);
    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>(32);

    let db = Arc::new(Mutex::new(Db::open()?));

    // Session manager: listens to events and persists to DB
    let db_clone = db.clone();
    let event_tx_clone = event_tx.clone();
    let mut event_rx = event_tx.subscribe();
    tokio::spawn(async move {
        session_manager(&mut event_rx, db_clone, event_tx_clone).await;
    });

    // Timer engine
    let timer = Timer::new(config);
    let event_tx_clone = event_tx.clone();
    tokio::spawn(async move {
        timer.run(event_tx_clone, cmd_rx).await;
    });

    // IPC server (blocks until process exits)
    ipc::start(event_tx, cmd_tx).await?;

    Ok(())
}

async fn session_manager(
    event_rx: &mut broadcast::Receiver<Event>,
    db: Arc<Mutex<Db>>,
    _event_tx: broadcast::Sender<Event>,
) {
    let mut current_session_id: Option<i64> = None;
    let mut session_started_at: Option<i64> = None;

    loop {
        let event = match event_rx.recv().await {
            Ok(e) => e,
            Err(broadcast::error::RecvError::Lagged(n)) => {
                log::warn!("session_manager lagged, dropped {} events", n);
                continue;
            }
            Err(broadcast::error::RecvError::Closed) => break,
        };

        match event {
            Event::Tick { phase, .. } if phase == "focus" && current_session_id.is_none() => {
                // First tick of a focus session — insert the session row
                let now = unix_now();
                let session = Session {
                    id: None,
                    started_at: now,
                    ended_at: None,
                    duration: None,
                    phase: Phase::Focus,
                    score: None,
                    journal: None,
                    interrupted: false,
                };
                match db.lock().unwrap().insert_session(&session) {
                    Ok(id) => {
                        current_session_id = Some(id);
                        session_started_at = Some(now);
                        log::info!("Session {} started", id);
                    }
                    Err(e) => log::error!("Failed to insert session: {}", e),
                }
            }

            Event::SessionEnd { score } => {
                if let (Some(id), Some(started)) = (current_session_id, session_started_at) {
                    let now = unix_now();
                    let duration = now - started;
                    let result = db.lock().unwrap().update_session_end(
                        id,
                        now,
                        duration,
                        score as i64,
                        None,
                        false,
                    );
                    if let Err(e) = result {
                        log::error!("Failed to update session: {}", e);
                    } else {
                        log::info!("Session {} ended, duration {}s, score {}", id, duration, score);
                    }
                    current_session_id = None;
                    session_started_at = None;
                }
            }

            Event::Tick { .. }
            | Event::BreakStart { .. }
            | Event::JournalPrompt
            | Event::StatusResponse { .. }
            | Event::Error { .. } => {}
        }
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}