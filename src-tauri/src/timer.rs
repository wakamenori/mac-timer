use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TimerStatus {
    Idle,
    Running,
    Paused,
    Finished,
}

#[derive(Debug, Clone, Serialize)]
pub struct BasicTimer {
    duration_secs: u32,
    remaining_secs: u32,
    status: TimerStatus,
}

impl BasicTimer {
    pub fn new(duration_secs: u32) -> Self {
        Self {
            duration_secs,
            remaining_secs: duration_secs,
            status: TimerStatus::Idle,
        }
    }

    pub fn remaining_secs(&self) -> u32 {
        self.remaining_secs
    }

    pub fn duration_secs(&self) -> u32 {
        self.duration_secs
    }

    pub fn status(&self) -> TimerStatus {
        self.status
    }

    pub fn is_finished(&self) -> bool {
        self.status == TimerStatus::Finished
    }

    pub fn tick(&mut self) {
        if self.status != TimerStatus::Running {
            return;
        }
        self.remaining_secs = self.remaining_secs.saturating_sub(1);
        if self.remaining_secs == 0 {
            self.status = TimerStatus::Finished;
        }
    }

    pub fn start(&mut self) {
        if self.status != TimerStatus::Finished {
            self.status = TimerStatus::Running;
        }
    }

    pub fn pause(&mut self) {
        if self.status == TimerStatus::Running {
            self.status = TimerStatus::Paused;
        }
    }

    pub fn reset(&mut self) {
        self.remaining_secs = self.duration_secs;
        self.status = TimerStatus::Idle;
    }

    pub fn set_duration(&mut self, secs: u32) {
        self.duration_secs = secs;
        self.remaining_secs = secs;
        self.status = TimerStatus::Idle;
    }

    pub fn display(&self) -> String {
        let total = self.remaining_secs;
        let h = total / 3600;
        let m = (total % 3600) / 60;
        let s = total % 60;
        if h > 0 {
            format!("{h}:{m:02}:{s:02}")
        } else {
            format!("{m:02}:{s:02}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_timer_has_full_remaining_seconds() {
        let timer = BasicTimer::new(300);
        assert_eq!(timer.remaining_secs(), 300);
    }

    #[test]
    fn tick_reduces_remaining_by_one() {
        let mut timer = BasicTimer::new(300);
        timer.start();
        timer.tick();
        assert_eq!(timer.remaining_secs(), 299);
    }

    #[test]
    fn remaining_never_goes_below_zero() {
        let mut timer = BasicTimer::new(1);
        timer.start();
        timer.tick();
        timer.tick();
        assert_eq!(timer.remaining_secs(), 0);
    }

    #[test]
    fn is_finished_when_remaining_is_zero() {
        let mut timer = BasicTimer::new(1);
        timer.start();
        timer.tick();
        assert!(timer.is_finished());
        assert_eq!(timer.status(), TimerStatus::Finished);
    }

    #[test]
    fn not_finished_when_remaining_is_positive() {
        let timer = BasicTimer::new(300);
        assert!(!timer.is_finished());
    }

    #[test]
    fn reset_restores_full_duration() {
        let mut timer = BasicTimer::new(300);
        timer.start();
        timer.tick();
        timer.tick();
        timer.reset();
        assert_eq!(timer.remaining_secs(), 300);
        assert_eq!(timer.status(), TimerStatus::Idle);
    }

    #[test]
    fn formats_as_minutes_and_seconds() {
        let timer = BasicTimer::new(125); // 2:05
        assert_eq!(timer.display(), "02:05");
    }

    #[test]
    fn formats_zero() {
        let timer = BasicTimer::new(0);
        assert_eq!(timer.display(), "00:00");
    }

    #[test]
    fn formats_with_hours_when_long() {
        let timer = BasicTimer::new(3661); // 1:01:01
        assert_eq!(timer.display(), "1:01:01");
    }

    #[test]
    fn paused_timer_does_not_tick() {
        let mut timer = BasicTimer::new(300);
        timer.start();
        timer.tick();
        assert_eq!(timer.remaining_secs(), 299);
        timer.pause();
        timer.tick();
        assert_eq!(timer.remaining_secs(), 299);
    }

    #[test]
    fn idle_timer_does_not_tick() {
        let mut timer = BasicTimer::new(300);
        timer.tick();
        assert_eq!(timer.remaining_secs(), 300);
    }

    #[test]
    fn set_duration_resets_timer() {
        let mut timer = BasicTimer::new(300);
        timer.start();
        timer.tick();
        timer.set_duration(600);
        assert_eq!(timer.remaining_secs(), 600);
        assert_eq!(timer.status(), TimerStatus::Idle);
    }
}
