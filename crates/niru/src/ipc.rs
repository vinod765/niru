use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;

use niru_core::ipc::{decode_event, encode_command, Command, Event};

pub struct IpcClient {
    pub cmd_tx: mpsc::Sender<Command>,
    pub event_rx: mpsc::Receiver<Event>,
}

impl IpcClient {
    pub async fn connect(socket_path: &str) -> anyhow::Result<Self> {
        let stream = UnixStream::connect(socket_path).await?;
        let (reader, mut writer) = stream.into_split();

        let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(16);
        let (event_tx, event_rx) = mpsc::channel::<Event>(64);

        // Write commands to daemon
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                match encode_command(&cmd) {
                    Ok(encoded) => {
                        if writer.write_all(encoded.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => log::warn!("Failed to encode command: {}", e),
                }
            }
        });

        // Read events from daemon
        tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                match decode_event(&line) {
                    Ok(event) => {
                        if event_tx.send(event).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => log::warn!("Bad event from daemon: {}", e),
                }
            }
        });

        Ok(Self { cmd_tx, event_rx })
    }

    pub async fn send(&self, cmd: Command) -> anyhow::Result<()> {
        self.cmd_tx.send(cmd).await?;
        Ok(())
    }
}