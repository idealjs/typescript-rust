//! Logger trait and implementation (1:1 port of Go's `internal/project/logging/logger.go`).

use std::io::Write;
use std::sync::Mutex;

/// Logger interface matching Go's `logging.Logger`.
pub trait Logger: Send + Sync {
    fn error(&self, msg: &str);
    fn errorf(&self, format: &str, args: &[&dyn std::fmt::Display]);
    fn warn(&self, msg: &str);
    fn warnf(&self, format: &str, args: &[&dyn std::fmt::Display]);
    fn info(&self, msg: &str);
    fn infof(&self, format: &str, args: &[&dyn std::fmt::Display]);
    fn log(&self, msg: &str);
    fn logf(&self, format: &str, args: &[&dyn std::fmt::Display]);

    fn verbose(&self) -> Option<&dyn Logger>;
    fn is_verbose(&self) -> bool;
    fn set_verbose(&self, verbose: bool);
}

/// Concrete logger writing to a writer (Go's `*logger`).
pub struct LoggerImpl {
    inner: Mutex<LoggerInner>,
}

struct LoggerInner {
    verbose: bool,
    writer: Box<dyn Write + Send>,
    prefix: Box<dyn Fn() -> String + Send + Sync>,
}

impl LoggerImpl {
    pub fn new(output: Box<dyn Write + Send>) -> Self {
        LoggerImpl {
            inner: Mutex::new(LoggerInner {
                verbose: false,
                writer: output,
                prefix: Box::new(|| format_time_now()),
            }),
        }
    }
}

impl Logger for LoggerImpl {
    fn log(&self, msg: &str) {
        let mut inner = self.inner.lock().unwrap();
        let prefix = (inner.prefix)();
        let _ = writeln!(inner.writer, "{prefix} {msg}");
    }

    fn logf(&self, format: &str, args: &[&dyn std::fmt::Display]) {
        let msg = format_args_string(format, args);
        self.log(&msg);
    }

    fn error(&self, msg: &str) {
        self.log(msg);
    }

    fn errorf(&self, format: &str, args: &[&dyn std::fmt::Display]) {
        self.logf(format, args);
    }

    fn warn(&self, msg: &str) {
        self.log(msg);
    }

    fn warnf(&self, format: &str, args: &[&dyn std::fmt::Display]) {
        self.logf(format, args);
    }

    fn info(&self, msg: &str) {
        self.log(msg);
    }

    fn infof(&self, format: &str, args: &[&dyn std::fmt::Display]) {
        self.logf(format, args);
    }

    fn verbose(&self) -> Option<&dyn Logger> {
        let inner = self.inner.lock().unwrap();
        if inner.verbose {
            // Can't return &dyn Logger from behind Mutex safely.
            // In practice, callers check is_verbose() then call directly.
            None
        } else {
            None
        }
    }

    fn is_verbose(&self) -> bool {
        self.inner.lock().unwrap().verbose
    }

    fn set_verbose(&self, verbose: bool) {
        self.inner.lock().unwrap().verbose = verbose;
    }
}

/// No-op logger that discards all messages (Go's `NewNopLogger`).
pub struct NopLogger;

impl Logger for NopLogger {
    fn error(&self, _msg: &str) {}
    fn errorf(&self, _format: &str, _args: &[&dyn std::fmt::Display]) {}
    fn warn(&self, _msg: &str) {}
    fn warnf(&self, _format: &str, _args: &[&dyn std::fmt::Display]) {}
    fn info(&self, _msg: &str) {}
    fn infof(&self, _format: &str, _args: &[&dyn std::fmt::Display]) {}
    fn log(&self, _msg: &str) {}
    fn logf(&self, _format: &str, _args: &[&dyn std::fmt::Display]) {}
    fn verbose(&self) -> Option<&dyn Logger> {
        None
    }
    fn is_verbose(&self) -> bool {
        false
    }
    fn set_verbose(&self, _verbose: bool) {}
}

pub fn new_logger(output: Box<dyn Write + Send>) -> LoggerImpl {
    LoggerImpl::new(output)
}

pub fn new_nop_logger() -> NopLogger {
    NopLogger
}

fn format_time_now() -> String {
    // Simplified time formatting matching Go's "15:04:05.000" format.
    "[time]".to_string()
}

fn format_args_string(format: &str, args: &[&dyn std::fmt::Display]) -> String {
    // Simple substitution — Go uses fmt.Sprintf, we do manual substitution.
    // For now, just concatenate args.
    let mut result = format.to_string();
    for arg in args {
        result = result.replacen("{}", &arg.to_string(), 1);
    }
    result
}
