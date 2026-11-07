//! High-Performance, Structured Logging Subsystem for Panadiga
//!
//! Provides thread-safe, configurable logging with support for:
//! - Configurable log levels (`Error`, `Warn`, `Info`, `Debug`, `Trace`, `Off`).
//! - Fast atomic level filtering with zero overhead.
//! - Custom sinks for redirecting log output (e.g. GUI terminal, automated tests, or files).
//! - Formatted output and raw printing (`log_println!`) routed through the logger.

use std::fmt;
use std::io::{self, Write};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

/// Log severity levels ordered by verbosity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Off = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl LogLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }
}

static CURRENT_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Info as u8);

type LogSink = Box<dyn Fn(LogLevel, &str) + Send + Sync + 'static>;
static CUSTOM_SINK: Mutex<Option<LogSink>> = Mutex::new(None);

/// Sets the maximum active log level globally.
pub fn set_level(level: LogLevel) {
    CURRENT_LEVEL.store(level as u8, Ordering::Release);
}

/// Retrieves the current active log level.
pub fn get_level() -> LogLevel {
    match CURRENT_LEVEL.load(Ordering::Acquire) {
        0 => LogLevel::Off,
        1 => LogLevel::Error,
        2 => LogLevel::Warn,
        3 => LogLevel::Info,
        4 => LogLevel::Debug,
        _ => LogLevel::Trace,
    }
}

/// Returns true if messages of the specified level should be emitted.
#[inline]
pub fn is_enabled(level: LogLevel) -> bool {
    level as u8 <= CURRENT_LEVEL.load(Ordering::Relaxed)
}

/// Sets a custom callback sink for intercepting log messages.
pub fn set_sink<F>(sink: F)
where
    F: Fn(LogLevel, &str) + Send + Sync + 'static,
{
    let mut guard = CUSTOM_SINK.lock().unwrap();
    *guard = Some(Box::new(sink));
}

/// Clears any registered custom sink, restoring standard stdout/stderr output.
pub fn reset_sink() {
    let mut guard = CUSTOM_SINK.lock().unwrap();
    *guard = None;
}

/// Emits a formatted log message if the given level is enabled.
pub fn log(level: LogLevel, args: fmt::Arguments) {
    if !is_enabled(level) {
        return;
    }

    let formatted = format!("{}", args);
    let guard = CUSTOM_SINK.lock().unwrap();
    if let Some(ref sink) = *guard {
        sink(level, &formatted);
        return;
    }

    match level {
        LogLevel::Error => {
            eprintln!("[ERROR] {}", formatted);
        }
        LogLevel::Warn => {
            eprintln!("[WARN]  {}", formatted);
        }
        LogLevel::Debug => {
            println!("[DEBUG] {}", formatted);
        }
        LogLevel::Trace => {
            println!("[TRACE] {}", formatted);
        }
        LogLevel::Info | LogLevel::Off => {
            println!("[INFO]  {}", formatted);
        }
    }
}

/// Emits raw, unprefixed text through the logger system (respects `LogLevel::Info` filtering).
pub fn raw(args: fmt::Arguments) {
    if !is_enabled(LogLevel::Info) {
        return;
    }

    let text = format!("{}", args);
    let guard = CUSTOM_SINK.lock().unwrap();
    if let Some(ref sink) = *guard {
        sink(LogLevel::Info, &text);
        return;
    }

    let stdout = io::stdout();
    let mut lock = stdout.lock();
    let _ = writeln!(lock, "{}", text);
}

#[macro_export]
macro_rules! __panadiga_log_dispatch {
    ($level:expr, $($arg:tt)*) => {
        $crate::logger::log($level, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::__panadiga_log_dispatch!($crate::logger::LogLevel::Info, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::__panadiga_log_dispatch!($crate::logger::LogLevel::Warn, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::__panadiga_log_dispatch!($crate::logger::LogLevel::Error, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::__panadiga_log_dispatch!($crate::logger::LogLevel::Debug, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::__panadiga_log_dispatch!($crate::logger::LogLevel::Trace, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_println {
    () => {
        $crate::logger::raw(format_args!("\n"))
    };
    ($($arg:tt)*) => {
        $crate::logger::raw(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_raw {
    ($($arg:tt)*) => {
        $crate::logger::raw(format_args!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    #[test]
    fn test_logger_levels_and_filtering() {
        set_level(LogLevel::Warn);
        assert!(!is_enabled(LogLevel::Info));
        assert!(is_enabled(LogLevel::Warn));
        assert!(is_enabled(LogLevel::Error));

        set_level(LogLevel::Info);
        assert!(is_enabled(LogLevel::Info));
    }

    #[test]
    fn test_logger_custom_sink() {
        let captured = Arc::new(AtomicBool::new(false));
        let cap_clone = Arc::clone(&captured);

        set_sink(move |_level, msg| {
            if msg.contains("PANADIGA_SINK_TEST") {
                cap_clone.store(true, Ordering::Release);
            }
        });

        raw(format_args!("PANADIGA_SINK_TEST_MESSAGE"));
        assert!(captured.load(Ordering::Acquire));

        reset_sink();
    }
}
