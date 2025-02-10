use core::time;
use std::{fmt::Display, time::Instant};

pub struct Perf {
    start: Instant,
    stop: Option<Instant>,
}

impl Perf {
    pub fn start() -> Self {
        Perf {
            start: Instant::now(),
            stop: None,
        }
    }
    pub fn stop(&mut self) {
        self.stop = Some(Instant::now());
    }
}

impl Display for Perf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.stop {
            Some(stop) => {
                write!(f, "{:?}", stop - self.start)
            }
            None => {
                write!(f, "started: {:?}", self.start)
            }
        }
    }
}
