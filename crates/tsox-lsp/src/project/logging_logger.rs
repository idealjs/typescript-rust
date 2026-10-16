use std::io::Write;
use std::sync::Mutex;

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

pub struct LoggerImpl {
    inner: Mutex<LoggerInner>,
}

struct LoggerInner {
    verbose: bool,
    writer: Box<dyn Write + Send>,
    prefix: Box<dyn Fn() -> String + Send + Sync>,
}

impl LoggerImpl {
    pub fn new(output: Box<dyn Write + Send>) -> Self { ::tsox_core::fntrace::enter("new"); 
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
    fn log(&self, msg: &str) { ::tsox_core::fntrace::enter("log"); 
        let mut inner = self.inner.lock().unwrap();
        let prefix = (inner.prefix)();
        let _ = writeln!(inner.writer, "{prefix} {msg}");
    }

    fn logf(&self, format: &str, args: &[&dyn std::fmt::Display]) { ::tsox_core::fntrace::enter("logf"); 
        let msg = format_args_string(format, args);
        self.log(&msg);
    }

    fn error(&self, msg: &str) { ::tsox_core::fntrace::enter("error"); 
        self.log(msg);
    }

    fn errorf(&self, format: &str, args: &[&dyn std::fmt::Display]) { ::tsox_core::fntrace::enter("errorf"); 
        self.logf(format, args);
    }

    fn warn(&self, msg: &str) { ::tsox_core::fntrace::enter("warn"); 
        self.log(msg);
    }

    fn warnf(&self, format: &str, args: &[&dyn std::fmt::Display]) { ::tsox_core::fntrace::enter("warnf"); 
        self.logf(format, args);
    }

    fn info(&self, msg: &str) { ::tsox_core::fntrace::enter("info"); 
        self.log(msg);
    }

    fn infof(&self, format: &str, args: &[&dyn std::fmt::Display]) { ::tsox_core::fntrace::enter("infof"); 
        self.logf(format, args);
    }

    fn verbose(&self) -> Option<&dyn Logger> { ::tsox_core::fntrace::enter("verbose"); 
        let inner = self.inner.lock().unwrap();
        if inner.verbose { None } else { None }
    }

    fn is_verbose(&self) -> bool { ::tsox_core::fntrace::enter("is_verbose"); 
        self.inner.lock().unwrap().verbose
    }

    fn set_verbose(&self, verbose: bool) { ::tsox_core::fntrace::enter("set_verbose"); 
        self.inner.lock().unwrap().verbose = verbose;
    }
}

pub struct NopLogger;

impl Logger for NopLogger {
    fn error(&self, _msg: &str) { ::tsox_core::fntrace::enter("error"); }
    fn errorf(&self, _format: &str, _args: &[&dyn std::fmt::Display]) { ::tsox_core::fntrace::enter("errorf"); }
    fn warn(&self, _msg: &str) { ::tsox_core::fntrace::enter("warn"); }
    fn warnf(&self, _format: &str, _args: &[&dyn std::fmt::Display]) { ::tsox_core::fntrace::enter("warnf"); }
    fn info(&self, _msg: &str) { ::tsox_core::fntrace::enter("info"); }
    fn infof(&self, _format: &str, _args: &[&dyn std::fmt::Display]) { ::tsox_core::fntrace::enter("infof"); }
    fn log(&self, _msg: &str) { ::tsox_core::fntrace::enter("log"); }
    fn logf(&self, _format: &str, _args: &[&dyn std::fmt::Display]) { ::tsox_core::fntrace::enter("logf"); }
    fn verbose(&self) -> Option<&dyn Logger> { ::tsox_core::fntrace::enter("verbose"); 
        None
    }
    fn is_verbose(&self) -> bool { ::tsox_core::fntrace::enter("is_verbose"); 
        false
    }
    fn set_verbose(&self, _verbose: bool) { ::tsox_core::fntrace::enter("set_verbose"); }
}

pub fn new_logger(output: Box<dyn Write + Send>) -> LoggerImpl { ::tsox_core::fntrace::enter("new_logger"); 
    LoggerImpl::new(output)
}

pub fn new_nop_logger() -> NopLogger { ::tsox_core::fntrace::enter("new_nop_logger"); 
    NopLogger
}

fn format_time_now() -> String { ::tsox_core::fntrace::enter("format_time_now"); 
    "[time]".to_string()
}

fn format_args_string(format: &str, args: &[&dyn std::fmt::Display]) -> String { ::tsox_core::fntrace::enter("format_args_string"); 
    let mut result = format.to_string();
    for arg in args {
        result = result.replacen("{}", &arg.to_string(), 1);
    }
    result
}
