use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Option<i64>,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub duration: Option<i64>,
    pub phase: Phase,
    pub score: Option<i64>,
    pub journal: Option<String>,
    pub interrupted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: Option<i64>,
    pub session_id: i64,
    pub timestamp: i64,
    pub events: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub session: SessionConfig,
    pub activity: ActivityConfig,
    pub sound: SoundConfig,
    pub notifications: NotificationsConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub base_duration: u32,
    pub max_extension: u32,
    pub short_break: u32,
    pub long_break: u32,
    pub long_break_after: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            base_duration: 25,
            max_extension: 15,
            short_break: 5,
            long_break: 20,
            long_break_after: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityConfig {
    pub idle_threshold: u32,
    pub sample_interval: u32,
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            idle_threshold: 30,
            sample_interval: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundConfig {
    pub enabled: bool,
    pub session_end: String,
    pub break_end: String,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            session_end: "~/.config/niru/sounds/end.ogg".into(),
            break_end: "~/.config/niru/sounds/start.ogg".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    pub enabled: bool,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { theme: "dark".into() }
    }
}
