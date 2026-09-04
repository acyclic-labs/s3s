use std::fmt;

/// Supplies wall-clock time for request authentication and policy validation.
///
/// Returning `None` makes time-dependent request processing fail closed. Services that do not
/// install a clock use [`SystemClock`].
pub trait Clock: Send + Sync + 'static {
    /// Returns the current UTC time, or `None` when authoritative time is unavailable.
    fn now_utc(&self) -> Option<time::OffsetDateTime>;
}

/// The default clock backed by the host wall clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_utc(&self) -> Option<time::OffsetDateTime> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Some(time::OffsetDateTime::now_utc())
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }
}

pub(crate) fn require_now_utc(clock: &dyn Clock) -> crate::S3Result<time::OffsetDateTime> {
    clock
        .now_utc()
        .ok_or_else(|| s3_error!(ServiceUnavailable, "server clock is unavailable"))
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Instant(std::time::Instant);

#[cfg(target_arch = "wasm32")]
pub struct Instant(());

impl Instant {
    pub fn now() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(std::time::Instant::now())
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self(())
        }
    }

    pub fn elapsed(&self) -> impl fmt::Debug {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0.elapsed()
        }
        #[cfg(target_arch = "wasm32")]
        {
            ()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_utc() {
        let t = SystemClock.now_utc();
        // Ensure it returns something Debug-formattable
        let _ = format!("{t:?}");
    }

    #[test]
    fn unavailable_clock_fails_closed() {
        struct UnavailableClock;

        impl Clock for UnavailableClock {
            fn now_utc(&self) -> Option<time::OffsetDateTime> {
                None
            }
        }

        let err = require_now_utc(&UnavailableClock).expect_err("unavailable time must fail");
        assert_eq!(*err.code(), crate::S3ErrorCode::ServiceUnavailable);
    }

    #[test]
    fn test_instant_now_and_elapsed() {
        let t = Instant::now();
        let e = t.elapsed();
        let _ = format!("{e:?}");
    }
}
