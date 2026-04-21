use std::{io::IsTerminal, sync::Mutex};

use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

static LOG_GUARD: Mutex<Option<WorkerGuard>> = Mutex::new(None);

pub fn initialize() {
    let log_dir = crate::path::local::get_logs_dir();
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer().json().with_writer(non_blocking);

    let console_layer = fmt::layer()
        .with_timer(fmt::time::uptime())
        .with_ansi(std::io::stderr().is_terminal())
        .with_writer(std::io::stderr)
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                EnvFilter::new("debug")
            } else {
                EnvFilter::new("info")
            }
        }));

    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .try_init()
        .ok();

    let mut g = LOG_GUARD.lock().unwrap();
    *g = Some(guard)
}

pub fn shutdown() {
    LOG_GUARD.lock().ok().and_then(|mut g| g.take());
}

pub fn with_logging<T>(f: impl FnOnce() -> anyhow::Result<T>) -> anyhow::Result<T> {
    initialize();
    let result = f();
    shutdown();
    result
}
