use std::time::{Duration, Instant};

const WORK_SECS: u64 = 25 * 60;
const SHORT_BREAK_SECS: u64 = 5 * 60;
const LONG_BREAK_SECS: u64 = 15 * 60;
const SESSIONS_BEFORE_LONG: u32 = 4;

#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn label(&self) -> &'static str {
        match self {
            Phase::Work => "WORK",
            Phase::ShortBreak => "SHORT BREAK",
            Phase::LongBreak => "LONG BREAK",
        }
    }

    pub fn duration_secs(&self) -> u64 {
        match self {
            Phase::Work => WORK_SECS,
            Phase::ShortBreak => SHORT_BREAK_SECS,
            Phase::LongBreak => LONG_BREAK_SECS,
        }
    }
}

pub struct Timer {
    pub phase: Phase,
    pub sessions_completed: u32,
    pub running: bool,
    started_at: Option<Instant>,
    elapsed_at_pause: Duration,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            phase: Phase::Work,
            sessions_completed: 0,
            running: false,
            started_at: None,
            elapsed_at_pause: Duration::ZERO,
        }
    }

    pub fn toggle(&mut self) {
        if self.running {
            self.elapsed_at_pause = self.elapsed();
            self.started_at = None;
            self.running = false;
        } else {
            self.started_at = Some(Instant::now());
            self.running = true;
        }
    }

    pub fn elapsed(&self) -> Duration {
        let live = self
            .started_at
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO);
        self.elapsed_at_pause + live
    }

    pub fn remaining(&self) -> Duration {
        let total = Duration::from_secs(self.phase.duration_secs());
        total.saturating_sub(self.elapsed())
    }

    pub fn is_finished(&self) -> bool {
        self.remaining() == Duration::ZERO
    }

    /// Advance to next phase. Returns true if a work session completed.
    pub fn advance(&mut self) -> bool {
        let completed_work = self.phase == Phase::Work;
        if completed_work {
            self.sessions_completed += 1;
        }
        self.phase = match self.phase {
            Phase::Work => {
                if self.sessions_completed % SESSIONS_BEFORE_LONG == 0 {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            Phase::ShortBreak | Phase::LongBreak => Phase::Work,
        };
        self.elapsed_at_pause = Duration::ZERO;
        self.started_at = if self.running {
            Some(Instant::now())
        } else {
            None
        };
        completed_work
    }

    pub fn reset(&mut self) {
        self.elapsed_at_pause = Duration::ZERO;
        self.started_at = if self.running {
            Some(Instant::now())
        } else {
            None
        };
    }

    pub fn format_remaining(&self) -> String {
        let secs = self.remaining().as_secs();
        format!("{:02}:{:02}", secs / 60, secs % 60)
    }

    pub fn progress(&self) -> f64 {
        let total = self.phase.duration_secs() as f64;
        let elapsed = self.elapsed().as_secs_f64();
        (elapsed / total).clamp(0.0, 1.0)
    }
}
