use std::io::{IsTerminal, stderr};
use std::sync::Mutex;

use anyhow::Result;
use fs_err as fs;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::{config, path};

mod types;
pub use types::*;

static LOG_GUARD: Mutex<Option<WorkerGuard>> = Mutex::new(None);

pub fn initialize() {
    let log_dir = path::local::get_logs_dir();
    fs::create_dir_all(&log_dir).ok();

    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_filter = config::get()
        .log
        .map(|x| EnvFilter::new(x.to_string()))
        .unwrap_or_else(|| EnvFilter::new(LogLevel::TRACE.to_string()));

    let console_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            LogLevel::TRACE.to_string().into()
        } else {
            LogLevel::OFF.to_string().into()
        }
    });

    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::ACTIVE)
        .with_filter(file_filter);

    let console_layer = fmt::layer()
        .with_timer(fmt::time::uptime())
        .with_ansi(stderr().is_terminal())
        .with_writer(stderr)
        .with_filter(console_filter);

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

pub fn with_logging<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    initialize();
    let result = f();
    shutdown();
    result
}
