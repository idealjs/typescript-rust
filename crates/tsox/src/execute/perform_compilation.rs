#![allow(unused_imports)]

use super::*;

pub(crate) fn perform_compilation(
    sys: &dyn System,
    config: ParsedCommandLine,
    pretty: bool,
    locale: Option<&Locale>,
) -> CommandLineResult {
    let host: Arc<dyn CompilerHost> = Arc::new(CompilerHostImpl::new(
        sys.fs(),
        sys.current_directory().to_string(),
        sys.default_library_path().to_string(),
    ));

    let program = Arc::new(Program::new(ProgramOptions {
        config,
        host: Arc::clone(&host),
    }));

    let diags = program.get_diagnostics_to_report();
    let mut writer = sys.writer();
    let error_count = report_diagnostics(&mut writer, &diags, pretty, locale).unwrap_or(0);

    let semantic_diags: Vec<Arc<Diagnostic>> = program
        .get_semantic_diagnostics()
        .into_iter()
        .map(Arc::new)
        .collect();
    let semantic_error_count = if !semantic_diags.is_empty() {
        let mut writer = sys.writer();
        report_diagnostics(&mut writer, &semantic_diags, pretty, locale).unwrap_or(0)
    } else {
        0
    };
    let error_count = error_count + semantic_error_count;

    let options = program.options();

    let should_emit = !options.no_emit.is_true()
        && !options.list_files_only.is_true()
        && (error_count == 0 || !options.no_emit_on_error.is_true());

    let _emitted_any;
    if should_emit {
        let fs = sys.fs();
        let emit_result = program.emit(&|path, data| {
            if let Some(parent) = std::path::Path::new(path).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            fs.write_file(path, data)
        });
        _emitted_any = !emit_result.emitted_files.is_empty();
        for diag in &emit_result.diagnostics {
            let _ = writeln!(writer, "{diag}");
        }
    }

    let status = if error_count > 0 {
        if !should_emit {
            ExitStatus::DiagnosticsPresent_OutputsSkipped
        } else {
            ExitStatus::DiagnosticsPresent_OutputsGenerated
        }
    } else {
        ExitStatus::Success
    };

    if options.list_files.is_true() || options.list_files_only.is_true() {
        for file in program.source_files() {
            let _ = writeln!(writer, "{}", file.file_name);
        }
    }

    CommandLineResult { status }
}

pub(crate) fn find_config_file(
    search_path: &str,
    file_exists: &dyn Fn(&str) -> bool,
    config_name: &str,
) -> String {
    let mut current = search_path.to_string();
    loop {
        let candidate = tspath::combine_paths(&current, &[config_name]);
        if file_exists(&candidate) {
            return candidate;
        }
        let parent = tspath::get_directory_path(&current);
        if parent == current {
            break;
        }
        current = parent;
    }
    String::new()
}

pub(crate) fn should_be_pretty(sys: &dyn System, options: &CompilerOptions) -> bool {
    match options.pretty {
        Tristate::True => true,
        Tristate::False => false,
        Tristate::Unknown => default_is_pretty(sys),
    }
}

pub(crate) fn default_is_pretty(sys: &dyn System) -> bool {
    if sys.environment_variable("NO_COLOR").is_some() {
        return false;
    }
    if sys.environment_variable("FORCE_COLOR").is_some() {
        return true;
    }
    sys.write_output_is_tty()
}

pub(crate) fn print_help(sys: &dyn System, show_all: bool) {
    let mut writer = sys.writer();
    let _ = writeln!(writer, "tsc: The TypeScript Compiler - Version {}", VERSION);
    let _ = writeln!(writer);

    if show_all {
        print_all_options_section(&mut writer);
    } else {
        print_simplified_help(&mut writer);
    }
}

pub(crate) fn print_simplified_help(writer: &mut dyn Write) {
    let _ = writeln!(writer, "COMMON COMMANDS:");
    let _ = writeln!(writer);
    let commands = [
        (
            "tsc",
            "Compile the current project (tsconfig.json in the working directory).",
        ),
        ("tsc app.ts util.ts", "Compile a set of .ts files."),
        ("tsc -b", "Build a composite project in build mode."),
        (
            "tsc --init",
            "Create a tsconfig.json in the current directory.",
        ),
        (
            "tsc -p ./path/to/tsconfig.json",
            "Compile a project at the given path.",
        ),
        ("tsc --help --all", "Show all compiler options."),
        ("tsc --noEmit", "Type-check without emitting output."),
        (
            "tsc --target esnext",
            "Compile to the latest ECMAScript target.",
        ),
    ];
    for (cmd, desc) in &commands {
        let _ = writeln!(writer, "  {cmd}");
        let _ = writeln!(writer, "    {desc}");
        let _ = writeln!(writer);
    }

    let mut cli_commands: Vec<&OptionDecl> = Vec::new();
    let mut config_opts: Vec<&OptionDecl> = Vec::new();
    for opt in OPTIONS.iter().filter(|o| o.show_in_simplified_help) {
        if opt.is_command_line_only {
            cli_commands.push(opt);
        } else {
            config_opts.push(opt);
        }
    }

    print_option_section(writer, "COMMAND LINE FLAGS:", &cli_commands);
    let _ = writeln!(writer);
    print_option_section(writer, "COMMON COMPILER OPTIONS:", &config_opts);
    let _ = writeln!(writer);
    let _ = writeln!(
        writer,
        "You can learn about all of the compiler options at https://aka.ms/tsc"
    );
}

pub(crate) fn print_all_options_section(writer: &mut dyn Write) {
    let mut compiler_opts: Vec<&OptionDecl> = OPTIONS
        .iter()
        .filter(|o| !o.description.is_empty())
        .collect();
    compiler_opts.sort_by_key(|o| o.name.to_lowercase());
    print_option_section(writer, "ALL COMPILER OPTIONS:", &compiler_opts);
    let _ = writeln!(writer);
    let _ = writeln!(
        writer,
        "You can learn about all of the compiler options at https://aka.ms/tsc"
    );
    let _ = writeln!(writer);

    let watch_opts: Vec<&OptionDecl> = OPTIONS_FOR_WATCH
        .iter()
        .filter(|o| !o.description.is_empty())
        .collect();
    print_option_section(writer, "WATCH OPTIONS:", &watch_opts);
    let _ = writeln!(writer);

    let build_opts: Vec<&OptionDecl> = BUILD_OPTIONS
        .iter()
        .filter(|o| o.name != "build" && !o.description.is_empty())
        .collect();
    print_option_section(writer, "BUILD OPTIONS:", &build_opts);
}

pub(crate) fn print_option_section(writer: &mut dyn Write, header: &str, opts: &[&OptionDecl]) {
    let _ = writeln!(writer, "{header}");
    let _ = writeln!(writer);
    if opts.is_empty() {
        return;
    }

    let name_strings: Vec<String> = opts.iter().map(|o| display_name_of_option(o)).collect();
    let max_name = name_strings.iter().map(|s| s.len()).max().unwrap_or(0);
    let col_width = max_name + 2;
    for (opt, name) in opts.iter().zip(name_strings.iter()) {
        if opt.description.is_empty() {
            let _ = writeln!(writer, "  {name}");
        } else {
            let _ = writeln!(writer, "  {name:<col_width$}{}", opt.description);
        }
    }
}

pub(crate) fn display_name_of_option(opt: &OptionDecl) -> String {
    match opt.short_name {
        Some(short) => format!("--{}, -{}", opt.name, short),
        None => format!("--{}", opt.name),
    }
}

pub(crate) fn write_config_file(sys: &dyn System, options: &CompilerOptions) -> CommandLineResult {
    let config_file_name = tspath::combine_paths(sys.current_directory(), &["tsconfig.json"]);
    if sys.fs().file_exists(&config_file_name) {
        let mut writer = sys.writer();
        let diag = compiler_diagnostic(
            A_TSCONFIG_JSON_FILE_IS_ALREADY_DEFINED_AT_COLON_0,
            vec![config_file_name.clone()],
        );
        let _ = writeln!(writer, "{}", format_diagnostic(&diag, false, None));
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    let config_text = generate_tsconfig(options);
    if let Err(err) = sys.fs().write_file(&config_file_name, &config_text) {
        let mut writer = sys.writer();
        let _ = writeln!(
            writer,
            "error TS5033: Could not write file '{config_file_name}': {err}."
        );
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    let mut writer = sys.writer();
    let _ = writeln!(writer);
    let _ = writeln!(writer, "Created a new tsconfig.json");
    let _ = writeln!(writer, "You can learn more at https://aka.ms/tsconfig");
    CommandLineResult {
        status: ExitStatus::Success,
    }
}
