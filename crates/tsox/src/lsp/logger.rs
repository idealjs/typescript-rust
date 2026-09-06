#![allow(dead_code)]

use std::io::Write;
use std::sync::Mutex;

use crate::lsp::lsproto;

pub fn max_verbosity_for_message_type(msg_type: lsproto::MessageType) -> lsproto::LogVerbosity {
    match msg_type {
        lsproto::MESSAGE_TYPE_ERROR => lsproto::LOG_VERBOSITY_ERROR,
        lsproto::MESSAGE_TYPE_WARNING => lsproto::LOG_VERBOSITY_WARNING,
        lsproto::MESSAGE_TYPE_INFO => lsproto::LOG_VERBOSITY_INFO,
        lsproto::MESSAGE_TYPE_DEBUG => lsproto::LOG_VERBOSITY_DEBUG,
        _ => lsproto::LOG_VERBOSITY_INFO,
    }
}

pub fn is_valid_log_verbosity(v: lsproto::LogVerbosity) -> bool {
    v >= lsproto::LOG_VERBOSITY_OFF && v <= lsproto::LOG_VERBOSITY_ERROR
}

pub struct Logger {
    verbosity: Mutex<lsproto::LogVerbosity>,
    init_started: Mutex<bool>,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            verbosity: Mutex::new(lsproto::LOG_VERBOSITY_INFO),
            init_started: Mutex::new(false),
        }
    }

    pub fn set_verbosity(&self, verbosity: lsproto::LogVerbosity) {
        *self.verbosity.lock().unwrap() = verbosity;
    }

    pub fn get_verbosity(&self) -> lsproto::LogVerbosity {
        *self.verbosity.lock().unwrap()
    }

    pub fn set_verbose(&self, verbose: bool) {
        let mut v = self.verbosity.lock().unwrap();
        *v = if verbose {
            lsproto::LOG_VERBOSITY_DEBUG
        } else {
            lsproto::LOG_VERBOSITY_INFO
        };
    }

    pub fn is_verbose(&self) -> bool {
        let v = *self.verbosity.lock().unwrap();
        v >= lsproto::LOG_VERBOSITY_TRACE && v <= lsproto::LOG_VERBOSITY_DEBUG
    }

    pub fn is_tracing(&self) -> bool {
        *self.verbosity.lock().unwrap() == lsproto::LOG_VERBOSITY_TRACE
    }

    pub fn mark_init_started(&self) {
        *self.init_started.lock().unwrap() = true;
    }

    pub fn is_init_started(&self) -> bool {
        *self.init_started.lock().unwrap()
    }

    pub fn send_log_message(&self, msg_type: lsproto::MessageType, message: &str) {
        let verbosity = *self.verbosity.lock().unwrap();
        if verbosity == lsproto::LOG_VERBOSITY_OFF
            || verbosity > max_verbosity_for_message_type(msg_type)
        {
            return;
        }

        let _ = writeln!(std::io::stderr(), "{}", message);
    }

    pub fn log(&self, msg: &str) {
        self.send_log_message(lsproto::MESSAGE_TYPE_INFO, msg);
    }

    pub fn logf(&self, format: &str, args: &[&str]) {

        let mut result = format.to_string();
        for arg in args {
            result = result.replacen("{}", arg, 1);
        }
        self.send_log_message(lsproto::MESSAGE_TYPE_INFO, &result);
    }

    pub fn error(&self, msg: &str) {
        self.send_log_message(lsproto::MESSAGE_TYPE_ERROR, msg);
    }

    pub fn warn(&self, msg: &str) {
        self.send_log_message(lsproto::MESSAGE_TYPE_WARNING, msg);
    }

    pub fn info(&self, msg: &str) {
        self.send_log_message(lsproto::MESSAGE_TYPE_INFO, msg);
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
