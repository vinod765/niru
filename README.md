# niru

> *From niruddha (निरुद्ध) — undistracted. Stay in flow.*

**Niru** is an adaptive, intelligent focus timer for Linux — built for developers who want more than a countdown clock.

It watches how you work, learns when you're sharp, flexes sessions when you're in flow, and gives you a real picture of your day — not just a pomodoro count.

---

## Why Niru?

Every Pomodoro app is a dumb timer. They don't know you. They interrupt your best moments. They count sessions but tell you nothing.

Niru is different:

- **Adaptive sessions** — detects keyboard/mouse activity and extends focus time when you're in flow
- **Micro-journal** — a 10-second prompt at the end of each session builds a log of what you actually did
- **Productivity heatmap** — shows *when* in the day you're genuinely sharp, built from your own data
- **Focus score** — not "6 pomodoros done" but a real picture of your day
- **Mood-aware breaks** — break length suggested based on how intense your session was
- **Zero mouse needed** — fully keyboard-driven TUI you summon and dismiss instantly

---

## Architecture

```
nirud  (daemon)          ←  runs silently in the background
  │
  │  Unix socket IPC
  │
niru   (TUI client)      ←  summon with a hotkey, glance, dismiss
```

Two binaries. Clean separation. Very Unix.

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the full breakdown.

---

## Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| TUI | ratatui |
| Config | TOML + serde |
| Storage | SQLite (rusqlite) |
| Notifications | notify-rust |
| Sound | rodio |
| IPC | Unix domain sockets |
| Activity tracking | /dev/input or evdev |

---

## Status

🚧 **Early development** — architecture and planning phase.

See [TODO.md](./TODO.md) for the full roadmap.

---

## Contributing

This is a personal project for now but will open up for contributions once the core is stable. Star the repo to follow along.

---

## License

MIT
