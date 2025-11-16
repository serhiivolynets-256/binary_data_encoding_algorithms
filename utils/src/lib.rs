use chrono::Local;
use std::fmt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::{format::Writer, time::FormatTime};
struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
        let now = Local::now();
        write!(w, "[{}]", now.format("%Y-%m-%d %H:%M:%S"))
    }
}

pub fn init_tracing() {
    _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_timer(LocalTimer)
        .with_target(true)
        .try_init();
}
