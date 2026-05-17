use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{broadcast, Mutex};

use niru_core::ipc::{decode_command, encode_event, Command, Event};

pub fn socket_path() -> String {
    let uid = unsafe { libc::getuid() };
    format!("/run/user/{}/nirud.sock", uid)
}

pub type EventTx = broadcast::Sender<Event>;
pub type CommandTx = tokio::sync::mpsc::Sender<Command>;

pub async fn start(event_tx: EventTx, cmd_tx: CommandTx) -> anyhow::Result<()> {
    let path = socket_path();

    if std::path::Path::new(&path).exists() {
        std::fs::remove_file(&path)?;
    }

    let listener = UnixListener::bind(&path)?;
    log::info!("IPC listening on {}", path);

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let event_rx = event_tx.subscribe();
                let cmd_tx = cmd_tx.clone();
                tokio::spawn(handle_client(stream, event_rx, cmd_tx));
            }
            Err(e) => {
                log::error!("Failed to accept IPC connection: {}", e);
            }
        }
    }
}

async fn handle_client(
    stream: UnixStream,
    mut event_rx: broadcast::Receiver<Event>,
    cmd_tx: CommandTx,
) {
    let (reader, writer) = stream.into_split();
    let writer = Arc::new(Mutex::new(writer));

    let writer_clone = writer.clone();
    let read_task = tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            match decode_command(&line) {
                Ok(cmd) => {
                    if cmd_tx.send(cmd).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let err_event = Event::Error {
                        message: format!("bad command: {}", e),
                    };
                    if let Ok(encoded) = encode_event(&err_event) {
                        let mut w = writer_clone.lock().await;
                        let _ = w.write_all(encoded.as_bytes()).await;
                    }
                }
            }
        }
    });

    let write_task = tokio::spawn(async move {
        loop {
            match event_rx.recv().await {
                Ok(event) => match encode_event(&event) {
                    Ok(encoded) => {
                        let mut w = writer.lock().await;
                        if w.write_all(encoded.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => log::error!("Failed to encode event: {}", e),
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    log::warn!("IPC client lagged, dropped {} events", n);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    tokio::select! {
        _ = read_task => {}
        _ = write_task => {}
    }

    log::info!("IPC client disconnected");
}