use rusqlite::{Connection, Result, params};
use std::path::PathBuf;

use niru_core::models::{ActivityLog, Phase, Session};

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open() -> Result<Self> {
        let path = db_path();
        std::fs::create_dir_all(path.parent().unwrap()).ok();
        let conn = Connection::open(&path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS sessions (
                id          INTEGER PRIMARY KEY,
                started_at  INTEGER NOT NULL,
                ended_at    INTEGER,
                duration    INTEGER,
                phase       TEXT,
                score       INTEGER,
                journal     TEXT,
                interrupted INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS activity_log (
                id          INTEGER PRIMARY KEY,
                session_id  INTEGER REFERENCES sessions(id),
                timestamp   INTEGER NOT NULL,
                events      INTEGER
            );
        ")
    }

    pub fn insert_session(&self, session: &Session) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO sessions (started_at, ended_at, duration, phase, score, journal, interrupted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                session.started_at,
                session.ended_at,
                session.duration,
                phase_to_str(&session.phase),
                session.score,
                session.journal,
                session.interrupted as i64,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_session_end(
        &self,
        id: i64,
        ended_at: i64,
        duration: i64,
        score: i64,
        journal: Option<&str>,
        interrupted: bool,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET ended_at=?1, duration=?2, score=?3, journal=?4, interrupted=?5
             WHERE id=?6",
            params![ended_at, duration, score, journal, interrupted as i64, id],
        )?;
        Ok(())
    }

    pub fn insert_activity(&self, log: &ActivityLog) -> Result<()> {
        self.conn.execute(
            "INSERT INTO activity_log (session_id, timestamp, events) VALUES (?1, ?2, ?3)",
            params![log.session_id, log.timestamp, log.events],
        )?;
        Ok(())
    }

    pub fn sessions_today(&self) -> Result<u32> {
        let midnight = today_midnight_unix();
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE started_at >= ?1 AND interrupted = 0",
            params![midnight],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn recent_sessions(&self, limit: u32) -> Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, started_at, ended_at, duration, phase, score, journal, interrupted
             FROM sessions ORDER BY started_at DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            Ok(Session {
                id: row.get(0)?,
                started_at: row.get(1)?,
                ended_at: row.get(2)?,
                duration: row.get(3)?,
                phase: str_to_phase(row.get::<_, String>(4)?.as_str()),
                score: row.get(5)?,
                journal: row.get(6)?,
                interrupted: row.get::<_, i64>(7)? != 0,
            })
        })?;

        rows.collect()
    }
}

fn db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    PathBuf::from(home).join(".local/share/niru/sessions.db")
}

fn today_midnight_unix() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    // truncate to day boundary (UTC)
    now - (now % 86400)
}

fn phase_to_str(phase: &Phase) -> &'static str {
    match phase {
        Phase::Focus => "focus",
        Phase::ShortBreak => "short_break",
        Phase::LongBreak => "long_break",
    }
}

fn str_to_phase(s: &str) -> Phase {
    match s {
        "short_break" => Phase::ShortBreak,
        "long_break" => Phase::LongBreak,
        _ => Phase::Focus,
    }
}