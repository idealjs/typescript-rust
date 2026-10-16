#![allow(unused_imports, dead_code)]

use std::io::Write;
use std::sync::{Arc, Mutex};

use tsox_core::core::compiler_options::CompilerOptions;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::diagnosticwriter;

use super::m5a::CommandLineTesting;
use crate::execute::System;
use tsox_frontend::diagnosticwriter::mig::m5z_2;
use tsox_frontend::diagnosticwriter::mig::x3::format_diagnostics_status_and_time;

pub type DiagnosticReporter = Box<dyn Fn(&Diagnostic) + Send + Sync>;
pub type DiagnosticsReporter = Box<dyn Fn(&[Arc<Diagnostic>]) + Send + Sync>;

pub use tsox_frontend::diagnosticwriter::mig::m5z::FormattingOptions;

pub fn get_format_opts_of_sys(
    sys: &dyn System,
    locale: &tsox_core::locale::Locale,
) -> FormattingOptions { ::tsox_core::fntrace::enter("get_format_opts_of_sys"); 
    FormattingOptions {
        locale: locale.clone(),
        compare_paths_options: tsox_core::tspath::ComparePathsOptions {
            current_directory: sys.current_directory().to_string(),
            use_case_sensitive_file_names: sys.fs().use_case_sensitive_file_names(),
        },
        new_line: "\n".to_string(),
    }
}

pub fn quiet_diagnostic_reporter(_diagnostic: &Diagnostic) { ::tsox_core::fntrace::enter("quiet_diagnostic_reporter"); }

pub fn create_diagnostic_reporter(
    sys: &dyn System,
    mut w: Box<dyn Write + Send + Sync>,
    locale: &tsox_core::locale::Locale,
    options: &CompilerOptions,
) -> DiagnosticReporter { ::tsox_core::fntrace::enter("create_diagnostic_reporter"); 
    if options.quiet.is_true() {
        return Box::new(quiet_diagnostic_reporter);
    }
    let format_opts = get_format_opts_of_sys(sys, locale);
    let w = Mutex::new(w);
    if crate::execute::perform_compilation::should_be_pretty(sys, options) {
        Box::new(move |diagnostic: &Diagnostic| {
            let mut w = w.lock().unwrap();
            m5z_2::format_diagnostic_with_color_and_context(
                &mut *w,
                diagnostic,
                &format_opts,
            );
            let _ = write!(w, "{}", format_opts.new_line);
        })
    } else {
        Box::new(move |diagnostic: &Diagnostic| {
            let mut w = w.lock().unwrap();
            m5z_2::write_format_diagnostic(&mut *w, diagnostic, &format_opts);
        })
    }
}

pub struct Colors {
    show_colors: bool,
    is_windows: bool,
    is_windows_terminal: bool,
    is_vs_code: bool,
    supports_richer_colors: bool,
}

pub fn create_colors(sys: &dyn System) -> Colors { ::tsox_core::fntrace::enter("create_colors"); 
    if !crate::execute::perform_compilation::default_is_pretty(sys) {
        return Colors {
            show_colors: false,
            is_windows: false,
            is_windows_terminal: false,
            is_vs_code: false,
            supports_richer_colors: false,
        };
    }
    let os = sys.environment_variable("OS").unwrap_or_default();
    let is_windows = os.to_lowercase().contains("windows");
    let wt_session = sys.environment_variable("WT_SESSION").unwrap_or_default();
    let term_program = sys.environment_variable("TERM_PROGRAM").unwrap_or_default();
    let color_term = sys.environment_variable("COLORTERM").unwrap_or_default();
    let term = sys.environment_variable("TERM").unwrap_or_default();
    Colors {
        show_colors: true,
        is_windows,
        is_windows_terminal: !wt_session.is_empty(),
        is_vs_code: term_program == "vscode",
        supports_richer_colors: color_term == "truecolor" || term == "xterm-256color",
    }
}

impl Colors {
    pub fn bold(&self, s: &str) -> String { ::tsox_core::fntrace::enter("bold"); 
        if !self.show_colors {
            return s.to_string();
        }
        format!("\x1b[1m{s}\x1b[22m")
    }

    pub fn blue(&self, s: &str) -> String { ::tsox_core::fntrace::enter("blue"); 
        if !self.show_colors {
            return s.to_string();
        }
        if self.is_windows && !self.is_windows_terminal && !self.is_vs_code {
            return self.bright_white(s);
        }
        format!("\x1b[94m{s}\x1b[39m")
    }

    pub fn blue_background(&self, s: &str) -> String { ::tsox_core::fntrace::enter("blue_background"); 
        if !self.show_colors {
            return s.to_string();
        }
        if self.supports_richer_colors {
            format!("\x1B[48;5;68m{s}\x1B[39;49m")
        } else {
            format!("\x1b[44m{s}\x1B[39;49m")
        }
    }

    pub fn bright_white(&self, s: &str) -> String { ::tsox_core::fntrace::enter("bright_white"); 
        if !self.show_colors {
            return s.to_string();
        }
        format!("\x1b[97m{s}\x1b[39m")
    }
}

pub fn quiet_diagnostics_reporter(_diagnostics: &[Arc<Diagnostic>]) { ::tsox_core::fntrace::enter("quiet_diagnostics_reporter"); }

pub fn create_report_error_summary(
    sys: &dyn System,
    locale: &tsox_core::locale::Locale,
    options: &CompilerOptions,
) -> DiagnosticsReporter { ::tsox_core::fntrace::enter("create_report_error_summary"); 
    if crate::execute::perform_compilation::should_be_pretty(sys, options) {
        let format_opts = get_format_opts_of_sys(sys, locale);
        let writer = Mutex::new(sys.writer());
        Box::new(move |diagnostics: &[Arc<Diagnostic>]| {
            let mut writer = writer.lock().unwrap();
            let plain: Vec<Diagnostic> = diagnostics.iter().map(|d| (**d).clone()).collect();
            m5z_2::write_error_summary_text(&mut *writer, &plain, &format_opts);
        })
    } else {
        Box::new(quiet_diagnostics_reporter)
    }
}

pub fn create_builder_status_reporter(
    sys: &dyn System,
    mut w: Box<dyn Write + Send + Sync>,
    locale: &tsox_core::locale::Locale,
    options: &CompilerOptions,
    testing: Option<Arc<dyn CommandLineTesting>>,
) -> DiagnosticReporter { ::tsox_core::fntrace::enter("create_builder_status_reporter"); 
    if options.quiet.is_true() {
        return Box::new(quiet_diagnostic_reporter);
    }
    let format_opts = get_format_opts_of_sys(sys, locale);
    let w = Mutex::new(w);
    Box::new(move |diagnostic: &Diagnostic| {
        let mut w = w.lock().unwrap();
        if let Some(testing) = testing.as_ref() {
            testing.on_build_status_report_start(&mut *w);
        }
        let time = format_time_of_now();
        format_diagnostics_status_and_time(&mut *w, &time, diagnostic, &format_opts);
        let _ = write!(w, "{}{}", format_opts.new_line, format_opts.new_line);
        if let Some(testing) = testing.as_ref() {
            testing.on_build_status_report_end(&mut *w);
        }
    })
}

pub fn create_watch_status_reporter(
    sys: &dyn System,
    locale: &tsox_core::locale::Locale,
    options: &CompilerOptions,
    testing: Option<Arc<dyn CommandLineTesting>>,
) -> DiagnosticReporter { ::tsox_core::fntrace::enter("create_watch_status_reporter"); 
    let format_opts = get_format_opts_of_sys(sys, locale);
    let options = options.clone();
    let writer = Mutex::new(sys.writer());
    let clear_screen =
        |writer: &mut dyn Write, diagnostic: &Diagnostic, options: &CompilerOptions| {
            m5z_2::try_clear_screen(writer, diagnostic, options)
        };
    Box::new(move |diagnostic: &Diagnostic| {
        let mut writer = writer.lock().unwrap();
        if let Some(testing) = testing.as_ref() {
            testing.on_watch_status_report_start();
        }
        clear_screen(&mut *writer, diagnostic, &options);
        let time = format_time_of_now();
        format_diagnostics_status_and_time(&mut *writer, &time, diagnostic, &format_opts);
        let _ = write!(writer, "{}{}", format_opts.new_line, format_opts.new_line);
        if let Some(testing) = testing.as_ref() {
            testing.on_watch_status_report_end();
        }
    })
}

fn format_time_of_now() -> String { ::tsox_core::fntrace::enter("format_time_of_now"); 
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let secs_of_day = secs % 86_400;
    let hour = (secs_of_day / 3_600) as usize;
    let minute = ((secs_of_day % 3_600) / 60) as usize;
    let second = (secs_of_day % 60) as usize;
    let (hour_12, am_pm) = match hour {
        0 => (12, "AM"),
        1..=11 => (hour, "AM"),
        12 => (12, "PM"),
        _ => (hour - 12, "PM"),
    };
    format!("{hour_12:02}:{minute:02}:{second:02} {am_pm}")
}
