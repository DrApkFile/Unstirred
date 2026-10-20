#![doc = "The time the store trusts. The engine and the store never read the operating system's \
clock themselves; the caller passes one in, so every expiry check is testable."]

use std::time::{SystemTime, UNIX_EPOCH};

/// The time the store trusts. The engine and the store never call the operating system; the
/// caller passes a clock, and only [`SystemClock`] reads one.
pub trait Clock: std::fmt::Debug {
    fn unix_seconds(&self) -> i64;
}

/// The clock the command line uses. A value that does not fit in the stored integer, or a clock
/// set before the unix epoch, becomes zero, which expires every challenge: the gate refuses
/// instead of admitting on a clock it cannot read.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn unix_seconds(&self) -> i64 {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(elapsed) => i64::try_from(elapsed.as_secs()).unwrap_or(0),
            Err(_) => 0,
        }
    }
}
