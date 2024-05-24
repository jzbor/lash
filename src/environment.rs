use core::fmt::Write;
use core::time::Duration;

pub trait EnvTimer {
    fn start() -> Self;
    fn end(self) -> Duration;
}

pub trait Environment: {
    type Timer: EnvTimer;

    fn stdout(&mut self) -> &mut impl Write;
    fn stderr(&mut self) -> &mut impl Write;

    fn start_timer(&self) -> Self::Timer {
        Self::Timer::start()
    }
}

#[cfg(feature = "std")]
pub struct StdEnvironment {
    stdout: StdStdout,
    stderr: StdStderr,
}

#[cfg(feature = "std")]
struct StdStdout {}

#[cfg(feature = "std")]
struct StdStderr {}

#[cfg(feature = "std")]
impl StdEnvironment {
    pub fn new() -> Self {
        StdEnvironment {
            stdout: StdStdout {},
            stderr: StdStderr {},
        }
    }
}

#[cfg(feature = "std")]
impl Environment for StdEnvironment {
    type Timer = std::time::Instant;

    fn stdout(&mut self) -> &mut impl Write {
        &mut self.stdout
    }

    fn stderr(&mut self) -> &mut impl Write {
        &mut self.stderr
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
