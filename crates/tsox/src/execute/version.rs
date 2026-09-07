#![allow(unused_imports)]

use super::*;

pub const VERSION: &str = "7.1.0-dev";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum ExitStatus {
    Success = 0,
    DiagnosticsPresent_OutputsSkipped = 1,
    DiagnosticsPresent_OutputsGenerated = 2,
    InvalidProject_OutputsSkipped = 3,
    ProjectReferenceCycle_OutputsSkipped = 4,
    NotImplemented = 5,
}

impl ExitStatus {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug)]
pub struct CommandLineResult {
    pub status: ExitStatus,
}

pub trait System: Send + Sync {
    fn writer(&self) -> Box<dyn Write + Send>;
    fn fs(&self) -> Arc<dyn FS>;
    fn default_library_path(&self) -> &str;
    fn current_directory(&self) -> &str;
    fn write_output_is_tty(&self) -> bool;
    fn width_of_terminal(&self) -> usize;
    fn environment_variable(&self, name: &str) -> Option<String>;
}

pub struct OsSystem {
    pub(crate) fs: Arc<BundledFS>,
    pub(crate) default_library_path: String,
    pub(crate) cwd: String,
    #[allow(dead_code)]
    pub(crate) start: Instant,
}

impl OsSystem {
    pub fn new() -> Self {
        let cwd = std::env::current_dir()
            .map(|p| tspath::normalize_path(&p.to_string_lossy()))
            .unwrap_or_else(|_| ".".to_string());
        Self {
            fs: Arc::new(BundledFS::new(Arc::new(OsFS))),
            default_library_path: bundled::lib_path(),
            cwd,
            start: Instant::now(),
        }
    }
}

impl Default for OsSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for OsSystem {
    fn writer(&self) -> Box<dyn Write + Send> {
        Box::new(std::io::stdout())
    }
    fn fs(&self) -> Arc<dyn FS> {
        Arc::clone(&self.fs) as Arc<dyn FS>
    }
    fn default_library_path(&self) -> &str {
        &self.default_library_path
    }
    fn current_directory(&self) -> &str {
        &self.cwd
    }
    fn write_output_is_tty(&self) -> bool {
        std::io::stdout().is_terminal()
    }
    fn width_of_terminal(&self) -> usize {
        80
    }
    fn environment_variable(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
}

pub(crate) fn compiler_diagnostic(
    message: crate::diagnostics::Message,
    args: Vec<String>,
) -> Diagnostic {
    Diagnostic::new(None, TextRange::undefined(), message, args)
}

pub(crate) fn locale_of(options: &CompilerOptions) -> Option<Locale> {
    if options.locale.is_empty() {
        None
    } else {
        Locale::parse(&options.locale)
    }
}

pub fn command_line(sys: &dyn System, args: &[String]) -> CommandLineResult {
    if let Some(first) = args.first() {
        if is_build_mode_arg(first) {
            let parsed =
                parse_build_command_line(args, sys.current_directory(), Some(sys.fs().as_ref()));
            return tsc_build_compilation(sys, parsed);
        }
    }

    if args.iter().skip(1).any(|arg| is_build_mode_arg(arg)) {
        let mut writer = sys.writer();
        let diag =
            compiler_diagnostic(OPTION_BUILD_MUST_BE_THE_FIRST_COMMAND_LINE_ARGUMENT, vec![]);
        let _ = writeln!(writer, "{}", format_diagnostic(&diag, false, None));
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    let parsed = parse_command_line(args, sys.current_directory(), Some(sys.fs().as_ref()));
    tsc_compilation(sys, parsed)
}

pub(crate) fn is_build_mode_arg(arg: &str) -> bool {
    matches!(
        arg.to_lowercase().as_str(),
        "-b" | "--b" | "-build" | "--build"
    )
}

pub(crate) fn tsc_build_compilation(
    sys: &dyn System,
    command_line: ParsedBuildCommandLine,
) -> CommandLineResult {
    let pretty = should_be_pretty(sys, &command_line.compiler_options);
    let locale = locale_of(&command_line.compiler_options);

    if !command_line.errors.is_empty() {
        let mut writer = sys.writer();
        for e in &command_line.errors {
            let _ = writeln!(writer, "{}", format_diagnostic(e, pretty, locale.as_ref()));
        }
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    if command_line.compiler_options.help.is_true() || command_line.compiler_options.all.is_true() {
        print_help(sys, command_line.compiler_options.all.is_true());
        return CommandLineResult {
            status: ExitStatus::Success,
        };
    }

    let projects = command_line.resolved_project_paths();

    let mut status = ExitStatus::Success;
    let mut seen_projects = HashSet::new();

    let mut building = HashSet::new();
    let mut cycle_stack: Vec<String> = Vec::new();
    for project in projects {
        let result = build_project(
            sys,
            &project,
            &command_line.compiler_options,
            &command_line.build_options,
            pretty,
            locale.as_ref(),
            &mut seen_projects,
            &mut building,
            &mut cycle_stack,
        );
        status = status.max(result.status);
    }

    CommandLineResult { status }
}
