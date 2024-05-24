use core::fmt::Write;
use core::time::Duration;

pub trait EnvTimer {
    fn start() -> Self;
    fn end(self) -> Duration;
}

pub trait Environment: {
    type Timer: EnvTimer;

    fn stdout(&self) -> impl Write;
    fn stderr(&self) -> impl Write;

    fn start_timer(&self) -> Self::Timer {
        Self::Timer::start()
    }
}

#[cfg(feature = "std")]
pub struct StdEnvironment {}

#[cfg(feature = "std")]
struct StdStdout {}

#[cfg(feature = "std")]
struct StdStderr {}

#[cfg(feature = "std")]
impl StdEnvironment {
    pub fn new() -> Self {
        StdEnvironment {}
    }
}

#[cfg(feature = "std")]
impl Environment for StdEnvironment {
    type Timer = std::time::Instant;

    fn stdout(&self) -> impl Write {
        StdStdout {}
    }

    fn stderr(&self) -> impl Write {
        StdStderr {}
    }
}

#[cfg(feature = "std")]
impl core::fmt::Write for StdStdout {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        println!("{}", s);
        Ok(())
    }
}

#[cfg(feature = "std")]
impl core::fmt::Write for StdStderr {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        eprintln!("{}", s);
        Ok(())
    }
}

#[cfg(feature = "std")]
impl EnvTimer for std::time::Instant {
    fn start() -> Self {
        std::time::Instant::now()
    }

    fn end(self) -> Duration {
        self.elapsed()
    }
}
