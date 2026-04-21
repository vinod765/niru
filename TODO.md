# Niru — Build Roadmap

A phased checklist. Work through these in order — each phase builds on the last.

---

## Phase 0 — Project Setup
- [ ] Initialize Cargo workspace with 3 crates: `nirud`, `niru`, `niru-core`
- [ ] Set up `niru-core` with shared models and IPC message types
- [ ] Add all dependencies to `Cargo.toml` files
- [ ] Verify `cargo build` works on a clean clone
- [ ] Set up `.gitignore` for Rust projects

---

## Phase 1 — Daemon Core (`nirud`)
- [ ] Basic timer engine (focus → short break → long break cycle)
- [ ] Config file loading from `~/.config/niru/config.toml`
- [ ] SQLite database setup — create tables on first run
- [ ] Session persistence — write session to DB on end
- [ ] Unix socket IPC server — accept connections, handle commands
- [ ] Graceful shutdown on SIGTERM/SIGINT
- [ ] Basic logging to `~/.local/share/niru/nirud.log`

---

## Phase 2 — TUI Client (`niru`)
- [ ] Unix socket IPC client — connect to daemon
- [ ] Basic ratatui app skeleton with key handling
- [ ] Live timer view — shows current phase + remaining time
- [ ] Start / pause / skip commands from TUI
- [ ] Status display (current streak, sessions today)
- [ ] Clean exit with `q`

---

## Phase 3 — Adaptive Sessions
- [ ] Activity monitor using `/dev/input` or `evdev` crate
- [ ] Idle detection — no input for N seconds = idle
- [ ] Flex logic — extend session when in flow
- [ ] Early end — cut session short when idle detected
- [ ] Activity logged to `activity_log` table per session

---

## Phase 4 — Focus Scoring
- [ ] Score algorithm (based on duration, activity, interruptions)
- [ ] Score stored with each session in DB
- [ ] Score displayed in TUI after each session ends
- [ ] Daily aggregate score

---

## Phase 5 — Micro Journal
- [ ] Journal prompt triggered at session end in TUI
- [ ] 10-second timeout — auto-skips if no input
- [ ] Journal text saved to session in DB
- [ ] Journal history view in TUI (scrollable log of past entries)

---

## Phase 6 — Heatmap Dashboard
- [ ] Query DB for hourly session data per day
- [ ] Render heatmap grid in ratatui (hours × days)
- [ ] Color intensity based on focus score
- [ ] Toggle between weekly and monthly view

---

## Phase 7 — Notifications + Sound
- [ ] Desktop notifications via `notify-rust` (session end, break end)
- [ ] Sound playback via `rodio` on session events
- [ ] Configurable sound file paths in config
- [ ] Toggle notifications and sound in config

---

## Phase 8 — Polish
- [ ] Config editor view in TUI (edit config without leaving the app)
- [ ] Mood-aware break suggestion (short/long based on session intensity)
- [ ] Multiple themes in TUI (dark/light)
- [ ] Systemd user service file for auto-starting `nirud`
- [ ] Man page for `niru` and `nirud`
- [ ] Installation script

---

## Phase 9 — Release
- [ ] README screenshots / demo GIF
- [ ] GitHub releases with pre-built binaries
- [ ] AUR package (Arch Linux)
- [ ] Announce on r/unixporn, r/rust, HackerNews

---

## Backlog (future ideas)
- [ ] Plugin system for custom session hooks
- [ ] Export session data to CSV
- [ ] Weekly summary email/report
- [ ] Wayland + X11 idle detection (libxss / ext-idle-notify)
- [ ] Web dashboard (optional, separate binary)
