use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PhaseTransition {
    pub from: Phase,
    pub to: Phase,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct PomodoroConfig {
    pub work_secs: u32,
    pub short_break_secs: u32,
    pub long_break_secs: u32,
    pub sessions_before_long_break: u32,
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_secs: 25 * 60,
            short_break_secs: 5 * 60,
            long_break_secs: 15 * 60,
            sessions_before_long_break: 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PomodoroStatus {
    Idle,
    Running,
    Paused,
}

#[derive(Debug, Clone, Serialize)]
pub struct PomodoroTimer {
    config: PomodoroConfig,
    phase: Phase,
    remaining_secs: u32,
    completed_sessions: u32,
    status: PomodoroStatus,
}

impl PomodoroTimer {
    pub fn new(config: PomodoroConfig) -> Self {
        Self {
            remaining_secs: config.work_secs,
            config,
            phase: Phase::Work,
            completed_sessions: 0,
            status: PomodoroStatus::Idle,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn remaining_secs(&self) -> u32 {
        self.remaining_secs
    }

    pub fn completed_sessions(&self) -> u32 {
        self.completed_sessions
    }

    pub fn status(&self) -> PomodoroStatus {
        self.status
    }

    pub fn start(&mut self) {
        self.status = PomodoroStatus::Running;
    }

    pub fn pause(&mut self) {
        if self.status == PomodoroStatus::Running {
            self.status = PomodoroStatus::Paused;
        }
    }

    pub fn reset(&mut self) {
        self.phase = Phase::Work;
        self.remaining_secs = self.config.work_secs;
        self.completed_sessions = 0;
        self.status = PomodoroStatus::Idle;
    }

    pub fn tick(&mut self) -> Option<PhaseTransition> {
        if self.status != PomodoroStatus::Running {
            return None;
        }
        self.remaining_secs = self.remaining_secs.saturating_sub(1);
        if self.remaining_secs == 0 {
            let from = self.phase;
            let to = match self.phase {
                Phase::Work => {
                    self.completed_sessions += 1;
                    if self
                        .completed_sessions
                        .is_multiple_of(self.config.sessions_before_long_break)
                    {
                        Phase::LongBreak
                    } else {
                        Phase::ShortBreak
                    }
                }
                Phase::ShortBreak | Phase::LongBreak => Phase::Work,
            };
            self.phase = to;
            self.remaining_secs = match to {
                Phase::Work => self.config.work_secs,
                Phase::ShortBreak => self.config.short_break_secs,
                Phase::LongBreak => self.config.long_break_secs,
            };
            Some(PhaseTransition { from, to })
        } else {
            None
        }
    }

    pub fn display(&self) -> String {
        let total = self.remaining_secs;
        let m = total / 60;
        let s = total % 60;
        format!("{m:02}:{s:02}")
    }

    pub fn session_display(&self) -> String {
        let total = self.config.sessions_before_long_break;
        (0..total)
            .map(|i| {
                if i < self.completed_sessions {
                    "●"
                } else {
                    "○"
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn tray_title(&self) -> String {
        let icon = match self.phase {
            Phase::Work => "🍅",
            Phase::ShortBreak | Phase::LongBreak => "☕",
        };
        format!("{icon} {}", self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_timer() -> PomodoroTimer {
        PomodoroTimer::new(PomodoroConfig::default())
    }

    fn fast_timer() -> PomodoroTimer {
        PomodoroTimer::new(PomodoroConfig {
            work_secs: 3,
            short_break_secs: 1,
            long_break_secs: 2,
            sessions_before_long_break: 4,
        })
    }

    #[test]
    fn starts_in_work_phase() {
        let timer = default_timer();
        assert_eq!(timer.phase(), Phase::Work);
    }

    #[test]
    fn work_phase_remaining_is_work_duration() {
        let timer = default_timer();
        assert_eq!(timer.remaining_secs(), 25 * 60);
    }

    #[test]
    fn work_transitions_to_short_break() {
        let mut timer = fast_timer();
        timer.start();
        // tick 3 times to finish work phase
        timer.tick();
        timer.tick();
        let transition = timer.tick();
        assert_eq!(
            transition,
            Some(PhaseTransition {
                from: Phase::Work,
                to: Phase::ShortBreak
            })
        );
        assert_eq!(timer.phase(), Phase::ShortBreak);
        assert_eq!(timer.remaining_secs(), 1);
    }

    #[test]
    fn work_transitions_to_long_break_after_4th() {
        let mut timer = fast_timer();
        timer.start();

        // Complete 3 work+short_break cycles
        for _ in 0..3 {
            // work (3 ticks)
            for _ in 0..3 {
                timer.tick();
            }
            // short break (1 tick)
            timer.tick();
        }

        // 4th work session (3 ticks)
        timer.tick();
        timer.tick();
        let transition = timer.tick();
        assert_eq!(
            transition,
            Some(PhaseTransition {
                from: Phase::Work,
                to: Phase::LongBreak
            })
        );
        assert_eq!(timer.phase(), Phase::LongBreak);
        assert_eq!(timer.remaining_secs(), 2);
    }

    #[test]
    fn long_break_transitions_back_to_work() {
        let mut timer = fast_timer();
        timer.start();

        // Complete 4 work sessions to get to long break
        for _ in 0..3 {
            for _ in 0..3 {
                timer.tick();
            }
            timer.tick();
        }
        for _ in 0..3 {
            timer.tick();
        }

        // Now in long break (2 ticks)
        assert_eq!(timer.phase(), Phase::LongBreak);
        timer.tick();
        let transition = timer.tick();
        assert_eq!(
            transition,
            Some(PhaseTransition {
                from: Phase::LongBreak,
                to: Phase::Work
            })
        );
        assert_eq!(timer.phase(), Phase::Work);
    }

    #[test]
    fn session_display_shows_dots() {
        let timer = default_timer();
        assert_eq!(timer.session_display(), "○ ○ ○ ○");
    }

    #[test]
    fn session_display_after_one_completed() {
        let mut timer = fast_timer();
        timer.start();
        for _ in 0..3 {
            timer.tick();
        }
        assert_eq!(timer.session_display(), "● ○ ○ ○");
    }

    #[test]
    fn tray_title_combines_icon_and_time() {
        let timer = default_timer();
        assert_eq!(timer.tray_title(), "🍅 25:00");
    }

    #[test]
    fn tray_title_during_break() {
        let mut timer = fast_timer();
        timer.start();
        for _ in 0..3 {
            timer.tick();
        }
        assert_eq!(timer.tray_title(), "☕ 00:01");
    }

    #[test]
    fn custom_config_uses_custom_durations() {
        let config = PomodoroConfig {
            work_secs: 50 * 60,
            short_break_secs: 10 * 60,
            long_break_secs: 30 * 60,
            sessions_before_long_break: 2,
        };
        let timer = PomodoroTimer::new(config);
        assert_eq!(timer.remaining_secs(), 50 * 60);
    }

    #[test]
    fn paused_timer_does_not_tick() {
        let mut timer = fast_timer();
        timer.start();
        timer.tick();
        assert_eq!(timer.remaining_secs(), 2);
        timer.pause();
        timer.tick();
        assert_eq!(timer.remaining_secs(), 2);
    }

    #[test]
    fn idle_timer_does_not_tick() {
        let mut timer = fast_timer();
        timer.tick();
        assert_eq!(timer.remaining_secs(), 3);
    }

    #[test]
    fn reset_restores_initial_state() {
        let mut timer = fast_timer();
        timer.start();
        for _ in 0..3 {
            timer.tick();
        }
        timer.reset();
        assert_eq!(timer.phase(), Phase::Work);
        assert_eq!(timer.remaining_secs(), 3);
        assert_eq!(timer.completed_sessions(), 0);
        assert_eq!(timer.status(), PomodoroStatus::Idle);
    }
}
