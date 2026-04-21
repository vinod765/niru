use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Command {
    Start,
    Pause,
    Skip,
    Stop,
    Status,
    Journal { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    Tick {
        remaining: u64,
        phase: String,
    },
    SessionEnd {
        score: u32,
    },
    BreakStart {
        duration: u64,
    },
    JournalPrompt,
    StatusResponse {
        phase: String,
        remaining: u64,
        streak: u32,
        sessions_today: u32,
    },
    Error {
        message: String,
    },
}

pub fn encode_command(cmd: &Command) -> serde_json::Result<String> {
    let mut s = serde_json::to_string(cmd)?;
    s.push('\n');
    Ok(s)
}

pub fn encode_event(event: &Event) -> serde_json::Result<String> {
    let mut s = serde_json::to_string(event)?;
    s.push('\n');
    Ok(s)
}

pub fn decode_command(s: &str) -> serde_json::Result<Command> {
    serde_json::from_str(s.trim())
}

pub fn decode_event(s: &str) -> serde_json::Result<Event> {
    serde_json::from_str(s.trim())
}
