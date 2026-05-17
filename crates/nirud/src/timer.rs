use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};

use niru_core::ipc::{Command, Event};
use niru_core::models::{Config, Phase};

#[derive(Debug, Clone, PartialEq)]
enum TimerState {
    Idle,
    Running,
    Paused,
}

pub struct Timer {
    state: TimerState,
    phase: Phase,
    remaining: u64,         // seconds
    sessions_completed: u32,
    config: Config,
    started_at: Option<Instant>,
}

impl Timer {
    pub fn new(config: Config) -> Self {
        let base = config.session.base_duration as u64 * 60;
        Self {
            state: TimerState::Idle,
            phase: Phase::Focus,
            remaining: base,
            sessions_completed: 0,
            config,
            started_at: None,
        }
    }

    fn focus_duration(&self) -> u64 {
        self.config.session.base_duration as u64 * 60
    }

    fn break_duration(&self) -> u64 {
        if self.sessions_completed > 0
            && self.sessions_completed % self.config.session.long_break_after == 0
        {
            self.config.session.long_break as u64 * 60
        } else {
            self.config.session.short_break as u64 * 60
        }
    }

    fn next_phase(&self) -> Phase {
        match self.phase {
            Phase::Focus => {
                if self.sessions_completed > 0
                    && self.sessions_completed % self.config.session.long_break_after == 0
                {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            Phase::ShortBreak | Phase::LongBreak => Phase::Focus,
        }
    }

    fn phase_label(&self) -> String {
        match self.phase {
            Phase::Focus => "focus".into(),
            Phase::ShortBreak => "short_break".into(),
            Phase::LongBreak => "long_break".into(),
        }
    }

    fn handle_command(&mut self, cmd: Command, event_tx: &broadcast::Sender<Event>) {
        match cmd {
            Command::Start => {
                if self.state != TimerState::Running {
                    if self.state == TimerState::Idle {
                        self.remaining = self.focus_duration();
                        self.phase = Phase::Focus;
                    }
                    self.state = TimerState::Running;
                    self.started_at = Some(Instant::now());
                    log::info!("Timer started: {:?} {}s", self.phase, self.remaining);
                }
            }
            Command::Pause => {
                if self.state == TimerState::Running {
                    self.state = TimerState::Paused;
                    log::info!("Timer paused");
                }
            }
            Command::Skip => {
                self.advance_phase(event_tx);
            }
            Command::Stop => {
                self.state = TimerState::Idle;
                self.remaining = self.focus_duration();
                self.phase = Phase::Focus;
                log::info!("Timer stopped");
            }
            Command::Status => {
                let _ = event_tx.send(Event::StatusResponse {
                    phase: self.phase_label(),
                    remaining: self.remaining,
                    streak: self.sessions_completed,
                    sessions_today: self.sessions_completed,
                });
            }
            Command::Journal { .. } => {
                // handled by session manager, not timer
            }
        }
    }

    fn advance_phase(&mut self, event_tx: &broadcast::Sender<Event>) {
        match self.phase {
            Phase::Focus => {
                self.sessions_completed += 1;
                let _ = event_tx.send(Event::SessionEnd {
                    score: 100, // placeholder until scorer is wired
                });

                let next = self.next_phase();
                let duration = self.break_duration();
                self.phase = next.clone();
                self.remaining = duration;
                self.state = TimerState::Running;

                let _ = event_tx.send(Event::BreakStart { duration });
                log::info!("Phase → {:?} for {}s", self.phase, duration);
            }
            Phase::ShortBreak | Phase::LongBreak => {
                self.phase = Phase::Focus;
                self.remaining = self.focus_duration();
                self.state = TimerState::Idle;
                log::info!("Break ended, ready for next focus session");
            }
        }
    }

    pub async fn run(
        mut self,
        event_tx: broadcast::Sender<Event>,
        mut cmd_rx: mpsc::Receiver<Command>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if self.state != TimerState::Running {
                        continue;
                    }

                    if self.remaining > 0 {
                        self.remaining -= 1;
                        let _ = event_tx.send(Event::Tick {
                            remaining: self.remaining,
                            phase: self.phase_label(),
                        });
                    } else {
                        self.advance_phase(&event_tx);
                    }
                }

                Some(cmd) = cmd_rx.recv() => {
                    self.handle_command(cmd, &event_tx);
                }
            }
        }
    }
}