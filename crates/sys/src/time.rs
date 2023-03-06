use std::time::Duration;

use crate::platform;
pub use platform::time::{schedule_wakeup, Instant, Interval, SystemTime};

pub async fn sleep(duration: Duration) {
    platform::time::sleep(duration).await
}

pub async fn sleep_until(instant: Instant) {
    platform::time::sleep_until(instant).await
}

pub fn interval(duration: Duration) -> Interval {
    Interval::new_at(Instant::now(), duration)
}
