extern crate alloc;

use alloc::string::String;
use core::fmt::Write;
use core::time::Duration;

use crate::error::*;

pub trait Environment: {
    type Instant;

    fn stdout(&mut self) -> &mut impl Write;
    fn stderr(&mut self) -> &mut impl Write;
    fn load(&self, file: &str) -> LashResult<String>;
    fn now(&self) -> Self::Instant;
    fn elapsed(&self, then: Self::Instant) -> Duration;
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
    type Instant = std::time::Instant;

    fn stdout(&mut self) -> &mut impl Write {
        &mut self.stdout
    }

    fn stderr(&mut self) -> &mut impl Write {
        &mut self.stderr
    }

    fn load(&self, file: &str) -> LashResult<String> {
        std::fs::read_to_string(&file)
            .map_err(|e| LashError::new_file_error(file.into(), Some(e)))
    }

    fn now(&self) -> Self::Instant {
        std::time::Instant::now()
    }

    fn elapsed(&self, then: Self::Instant) -> Duration {
        then.elapsed()
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
