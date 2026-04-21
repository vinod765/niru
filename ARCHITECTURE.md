# Niru — Architecture

## Overview

Niru follows the Unix philosophy: do one thing well, separate concerns cleanly.

Two binaries:
- **`nirud`** — a background daemon that owns all state, timing, and data
- **`niru`** — a TUI client that connects to the daemon, displays data, and sends commands

They communicate over a **Unix domain socket** at `/run/user/$UID/nirud.sock`.

---

## System Diagram

```
┌─────────────────────────────────────────────────────┐
│                    nirud  (daemon)                  │
│                                                     │
│  ┌─────────────┐   ┌──────────────┐   ┌─────────┐  │
│  │ Timer Engine│   │Activity Watch│   │ Scorer  │  │
│  │             │   │ (evdev/)     │   │         │  │
│  │ • session   │   │ • keystrokes │   │ • focus │  │
│  │ • break     │   │ • mouse move │   │   score │  │
│  │ • flex logic│   │ • idle detect│   │ • streak│  │
│  └──────┬──────┘   └──────┬───────┘   └────┬────┘  │
│         │                 │                │        │
│  ┌──────▼─────────────────▼────────────────▼─────┐  │
│  │              Session Manager                  │  │
│  │   • persists sessions to SQLite               │  │
│  │   • triggers notifications + sound            │  │
│  │   • manages micro-journal prompts             │  │
│  └──────────────────────┬────────────────────────┘  │
│                         │                           │
│                    SQLite DB                        │
│            ~/.local/share/niru/sessions.db          │
└─────────────────────────┬───────────────────────────┘
                          │
                  Unix Socket IPC
              /run/user/$UID/nirud.sock
                          │
┌─────────────────────────▼───────────────────────────┐
│                   niru  (TUI client)                │
│                                                     │
│  ┌──────────────┐  ┌─────────────┐  ┌────────────┐ │
│  │  Live Timer  │  │  Heatmap    │  │  Journal   │ │
│  │  View        │  │  Dashboard  │  │  History   │ │
│  └──────────────┘  └─────────────┘  └────────────┘ │
│                                                     │
│  ┌──────────────┐  ┌─────────────┐                 │
│  │ Focus Score  │  │   Config    │                 │
│  │ Panel        │  │   Editor    │                 │
│  └──────────────┘  └─────────────┘                 │
└─────────────────────────────────────────────────────┘
```

---

## IPC Protocol

Simple JSON messages over the Unix socket.

### Commands (client → daemon)
```json
{ "cmd": "start" }
{ "cmd": "pause" }
{ "cmd": "skip" }
{ "cmd": "stop" }
{ "cmd": "status" }
{ "cmd": "journal", "text": "implemented auth flow" }
```

### Events (daemon → client, streamed)
```json
{ "event": "tick", "remaining": 1234, "phase": "focus" }
{ "event": "session_end", "score": 87 }
{ "event": "break_start", "duration": 300 }
{ "event": "journal_prompt" }
```

---

## Data Model

### sessions table
```sql
CREATE TABLE sessions (
    id          INTEGER PRIMARY KEY,
    started_at  INTEGER NOT NULL,   -- unix timestamp
    ended_at    INTEGER,
    duration    INTEGER,            -- seconds actually focused
    phase       TEXT,               -- 'focus' | 'short_break' | 'long_break'
    score       INTEGER,            -- 0-100 focus score
    journal     TEXT,               -- micro-journal entry
    interrupted INTEGER DEFAULT 0  -- was it cut short?
);
```

### activity_log table
```sql
CREATE TABLE activity_log (
    id          INTEGER PRIMARY KEY,
    session_id  INTEGER REFERENCES sessions(id),
    timestamp   INTEGER NOT NULL,
    events      INTEGER             -- input events in this window
);
```

---

## Adaptive Session Logic

The core innovation. Instead of a rigid 25-minute timer:

1. Every 30 seconds, `nirud` samples input activity
2. If activity is **above threshold** → you're in flow → silently extend the session by 2 minutes (up to a max cap)
3. If activity **drops to idle** → you've naturally finished → end session early, log it
4. Score is calculated from: actual focus time, activity consistency, interruptions

---

## Config (`~/.config/niru/config.toml`)

```toml
[session]
base_duration = 25        # minutes
max_extension = 15        # max flex minutes added
short_break = 5
long_break = 20
long_break_after = 4      # sessions before long break

[activity]
idle_threshold = 30       # seconds of inactivity = idle
sample_interval = 30      # seconds between activity checks

[sound]
enabled = true
session_end = "~/.config/niru/sounds/end.ogg"
break_end   = "~/.config/niru/sounds/start.ogg"

[notifications]
enabled = true

[ui]
theme = "dark"            # dark | light
```

---

## File Structure

```
niru/
├── Cargo.toml              # workspace
├── crates/
│   ├── nirud/              # daemon binary
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── timer.rs        # session + flex logic
│   │   │   ├── activity.rs     # input monitoring
│   │   │   ├── scorer.rs       # focus score
│   │   │   ├── db.rs           # sqlite
│   │   │   ├── ipc.rs          # unix socket server
│   │   │   ├── notify.rs       # desktop notifications
│   │   │   └── sound.rs        # audio
│   │   └── Cargo.toml
│   ├── niru/               # TUI client binary
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── ipc.rs          # unix socket client
│   │   │   ├── ui/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── timer.rs    # live timer view
│   │   │   │   ├── heatmap.rs  # productivity heatmap
│   │   │   │   ├── journal.rs  # session history
│   │   │   │   ├── score.rs    # focus score panel
│   │   │   │   └── config.rs   # config editor
│   │   └── Cargo.toml
│   └── niru-core/          # shared types
│       ├── src/
│       │   ├── lib.rs
│       │   ├── models.rs   # Session, ActivityLog, Config
│       │   └── ipc.rs      # shared message types
│       └── Cargo.toml
├── README.md
├── ARCHITECTURE.md
└── TODO.md
```
