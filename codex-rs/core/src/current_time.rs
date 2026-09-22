use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use chrono::DateTime;
use chrono::Utc;
use codex_features::CurrentTimeSource;
use codex_protocol::ThreadId;

use crate::config::CurrentTimeReminderConfig;

pub type TimeFuture<'a> = Pin<Box<dyn Future<Output = Result<DateTime<Utc>>> + Send + 'a>>;
pub type SleepFuture<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

/// Host integration boundary for reading and waiting on the current time.
pub trait TimeProvider: Send + Sync {
    fn current_time(&self, thread_id: ThreadId) -> TimeFuture<'_>;

    /// Waits for the given duration on this provider's clock.
    ///
    /// Dropping the returned future cancels the wait.
    fn sleep(&self, thread_id: ThreadId, duration: Duration) -> SleepFuture<'_>;
}

pub(crate) struct SystemTimeProvider;

/// Pin the clock so a measurement run can reproduce an older run's prompt exactly.
///
/// The environment context carries `<current_date>`, so two runs on different days never
/// send the same bytes no matter what else is held equal -- which makes a before/after
/// comparison across a date boundary measure the calendar as well as the change. Set
/// `SUFFICE_FAKE_DATE=YYYY-MM-DD` to fix it; unset, nothing changes, and no code path
/// reads it unless the variable is present.
///
/// This is for measurement, not for use: the model is told a date that is not today.
pub(crate) fn faked_now() -> Option<chrono::DateTime<Utc>> {
    let raw = std::env::var("SUFFICE_FAKE_DATE").ok()?;
    let date = chrono::NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").ok()?;
    Some(chrono::DateTime::from_naive_utc_and_offset(
        date.and_hms_opt(12, 0, 0)?,
        Utc,
    ))
}

impl TimeProvider for SystemTimeProvider {
    fn current_time(&self, _thread_id: ThreadId) -> TimeFuture<'_> {
        Box::pin(async { Ok(faked_now().unwrap_or_else(Utc::now)) })
    }

    fn sleep(&self, _thread_id: ThreadId, duration: Duration) -> SleepFuture<'_> {
        Box::pin(async move {
            tokio::time::sleep(duration).await;
            Ok(())
        })
    }
}

pub(crate) fn resolve_time_provider(
    config: Option<&CurrentTimeReminderConfig>,
    external_provider: Option<Arc<dyn TimeProvider>>,
) -> Result<Arc<dyn TimeProvider>> {
    match config.map(|config| config.clock_source).unwrap_or_default() {
        CurrentTimeSource::System => Ok(Arc::new(SystemTimeProvider)),
        CurrentTimeSource::External => external_provider.ok_or_else(|| {
            anyhow!(
                "features.current_time_reminder.clock_source is external, but no external current-time provider is available"
            )
        }),
    }
}
