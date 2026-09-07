#![allow(unused_imports)]

use super::*;

pub(crate) fn tsc_compilation(
    sys: &dyn System,
    command_line: ParsedCommandLine,
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

    let options = &command_line.compiler_options;

    if options.init.is_true() {
        return write_config_file(sys, options);
    }

    if options.version.is_true() {
        let mut writer = sys.writer();
        let _ = writeln!(writer, "Version {}", VERSION);
        return CommandLineResult {
            status: ExitStatus::Success,
        };
    }

    if options.help.is_true() || options.all.is_true() {
        print_help(sys, options.all.is_true());
        return CommandLineResult {
            status: ExitStatus::Success,
        };
    }

    if options.watch.is_true() && options.list_files_only.is_true() {
        let mut writer = sys.writer();
        let diag = compiler_diagnostic(
            OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
            vec!["watch".to_string(), "listFilesOnly".to_string()],
        );
        let _ = writeln!(
            writer,
            "{}",
            format_diagnostic(&diag, pretty, locale.as_ref())
        );
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    let mut config_file_name = String::new();

    if !options.project.is_empty() {
        if !command_line.file_names.is_empty() {
            let mut writer = sys.writer();
            let diag = compiler_diagnostic(
                OPTION_PROJECT_CANNOT_BE_MIXED_WITH_SOURCE_FILES_ON_A_COMMAND_LINE,
                vec![],
            );
            let _ = writeln!(
                writer,
                "{}",
                format_diagnostic(&diag, pretty, locale.as_ref())
            );
            return CommandLineResult {
                status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
            };
        }
        let file_or_directory = tspath::normalize_path(&tspath::combine_paths(
            sys.current_directory(),
            &[&options.project],
        ));
        if sys.fs().directory_exists(&file_or_directory) {
            config_file_name = tspath::combine_paths(&file_or_directory, &["tsconfig.json"]);
            if !sys.fs().file_exists(&config_file_name) {
                let mut writer = sys.writer();
                let diag = compiler_diagnostic(
                    CANNOT_FIND_A_TSCONFIG_JSON_FILE_AT_THE_SPECIFIED_DIRECTORY_COLON_0,
                    vec![config_file_name.clone()],
                );
                let _ = writeln!(
                    writer,
                    "{}",
                    format_diagnostic(&diag, pretty, locale.as_ref())
                );
                return CommandLineResult {
                    status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
                };
            }
        } else {
            config_file_name = file_or_directory.clone();
            if !sys.fs().file_exists(&config_file_name) {
                let mut writer = sys.writer();
                let diag = compiler_diagnostic(
                    THE_SPECIFIED_PATH_DOES_NOT_EXIST_COLON_0,
                    vec![file_or_directory.clone()],
                );
                let _ = writeln!(
                    writer,
                    "{}",
                    format_diagnostic(&diag, pretty, locale.as_ref())
                );
                return CommandLineResult {
                    status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
                };
            }
        }
    } else if !options.ignore_config.is_true() || command_line.file_names.is_empty() {
        let search_path = tspath::normalize_path(sys.current_directory());
        config_file_name =
            find_config_file(&search_path, &|p| sys.fs().file_exists(p), "tsconfig.json");
        if !command_line.file_names.is_empty() {
            if !config_file_name.is_empty() {
                let mut writer = sys.writer();
                let diag = compiler_diagnostic(
                    X_TSCONFIG_JSON_IS_PRESENT_BUT_WILL_NOT_BE_LOADED_IF_FILES_ARE_SPECIFIED_ON_COMMANDLINE_USE_IGNORECONFIG_TO_SKIP_THIS_ERROR,
                    vec![],
                );
                let _ = writeln!(
                    writer,
                    "{}",
                    format_diagnostic(&diag, pretty, locale.as_ref())
                );
                return CommandLineResult {
                    status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
                };
            }
        } else if config_file_name.is_empty() {
            let mut writer = sys.writer();
            let diag = compiler_diagnostic(
                CANNOT_FIND_A_TSCONFIG_JSON_FILE_AT_THE_CURRENT_DIRECTORY_COLON_0,
                vec![search_path],
            );
            let _ = writeln!(
                writer,
                "{}",
                format_diagnostic(&diag, pretty, locale.as_ref())
            );
            let _ = writeln!(writer, "  Searching for: tsconfig.json");
            print_help(sys, false);
            return CommandLineResult {
                status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
            };
        }
    }

    let show_config_requested = command_line.compiler_options.show_config.is_true();

    let base_options = command_line.compiler_options.clone();

    let config_for_compilation: ParsedCommandLine = if !config_file_name.is_empty() {
        let config_parsed = get_parsed_command_line_of_config_file(
            &config_file_name,
            &command_line.compiler_options,
            sys.current_directory(),
            sys.fs().as_ref(),
        );
        if !config_parsed.errors.is_empty() {
            let mut writer = sys.writer();
            for e in &config_parsed.errors {
                let _ = writeln!(writer, "{}", format_diagnostic(e, pretty, locale.as_ref()));
            }
            return CommandLineResult {
                status: ExitStatus::DiagnosticsPresent_OutputsGenerated,
            };
        }
        config_parsed
    } else {
        command_line
    };

    if show_config_requested {
        show_config(sys, &config_for_compilation);
        return CommandLineResult {
            status: ExitStatus::Success,
        };
    }

    if config_for_compilation.compiler_options.watch.is_true() {
        return watch::watch_mode(
            sys,
            config_for_compilation,
            base_options,
            &config_file_name,
            pretty,
            locale,
        );
    }

    perform_compilation(sys, config_for_compilation, pretty, locale.as_ref())
}
