#![allow(unused_imports, dead_code)]

use std::io::Write;

use tsox_core::diagnostics::Message;
use tsox_core::locale::Locale;
use tsox_tsoptions::tsoptions::{OptionDecl, OptionKind, BUILD_OPTIONS, OPTIONS, OPTIONS_FOR_WATCH};

use super::m5a_4::{Colors, create_colors};
use super::m5a_6::{generate_option_output, get_display_name_text_of_option};
use crate::execute::System;

pub fn print_version(sys: &dyn System, locale: &Locale) {
    let mut writer = sys.writer();
    let _ = writeln!(
        writer,
        "{}",
        tsox_core::diagnostics::VERSION_0.localize(locale, &[&tsox_core::core::mig::m3k::version()])
    );
}

pub fn get_options_for_help(all: bool) -> Vec<&'static OptionDecl> {
    let mut opts: Vec<&'static OptionDecl> = OPTIONS.iter().chain(BUILD_OPTIONS.iter()).collect();
    if all {
        opts.sort_by_key(|o| o.name.to_lowercase());
        opts
    } else {
        opts.into_iter().filter(|o| o.show_in_simplified_help).collect()
    }
}

pub fn get_header(sys: &dyn System, colors: &Colors, message: &str) -> Vec<String> {
    let mut header: Vec<String> = Vec::with_capacity(3);
    let terminal_width = sys.width_of_terminal();
    const TS_ICON: &str = "     ";
    const TS_ICON_TS: &str = "  TS ";
    let ts_icon_length = TS_ICON.len();

    let ts_icon_first_line = colors.blue_background(TS_ICON);
    let ts_icon_second_line = colors.blue_background(&colors.bright_white(TS_ICON_TS));
    if terminal_width >= message.len() + ts_icon_length {
        let right_align = if terminal_width > 120 { 120 } else { terminal_width };
        let left_align = right_align - ts_icon_length;
        header.push(format!("{message:<left_align$}"));
        header.push(ts_icon_first_line);
        header.push("\n".to_string());
        header.push(" ".repeat(left_align));
        header.push(ts_icon_second_line);
        header.push("\n".to_string());
    } else {
        header.push(message.to_string());
        header.push("\n".to_string());
        header.push("\n".to_string());
    }
    header
}

pub fn print_easy_help(sys: &dyn System, locale: &Locale, simple_options: &[&OptionDecl]) {
    let colors = create_colors(sys);
    let mut output: Vec<String> = Vec::new();

    let msg = format!(
        "{} - {}",
        tsox_core::diagnostics::X_TSC_COLON_THE_TYPESCRIPT_COMPILER.localize(locale, &[]),
        tsox_core::diagnostics::VERSION_0.localize(locale, &[&tsox_core::core::mig::m3k::version()])
    );
    output.extend(get_header(sys, &colors, &msg));
    output.push(colors.bold(&tsox_core::diagnostics::COMMON_COMMANDS.localize(locale, &[])));
    output.push("\n".to_string());
    output.push("\n".to_string());

    let examples: &[(&[&str], &Message)] = &[
        (&["tsc"], &tsox_core::diagnostics::COMPILES_THE_CURRENT_PROJECT_TSCONFIG_JSON_IN_THE_WORKING_DIRECTORY),
        (&["tsc app.ts util.ts"], &tsox_core::diagnostics::IGNORING_TSCONFIG_JSON_COMPILES_THE_SPECIFIED_FILES_WITH_DEFAULT_COMPILER_OPTIONS),
        (&["tsc -b"], &tsox_core::diagnostics::BUILD_A_COMPOSITE_PROJECT_IN_THE_WORKING_DIRECTORY),
        (&["tsc --init"], &tsox_core::diagnostics::CREATES_A_TSCONFIG_JSON_WITH_THE_RECOMMENDED_SETTINGS_IN_THE_WORKING_DIRECTORY),
        (&["tsc -p ./path/to/tsconfig.json"], &tsox_core::diagnostics::COMPILES_THE_TYPESCRIPT_PROJECT_LOCATED_AT_THE_SPECIFIED_PATH),
        (&["tsc --help --all"], &tsox_core::diagnostics::AN_EXPANDED_VERSION_OF_THIS_INFORMATION_SHOWING_ALL_POSSIBLE_COMPILER_OPTIONS),
        (&["tsc --noEmit", "tsc --target esnext"], &tsox_core::diagnostics::COMPILES_THE_CURRENT_PROJECT_WITH_ADDITIONAL_SETTINGS),
    ];
    for (cmds, desc) in examples {
        for example in *cmds {
            output.push("  ".to_string());
            output.push(colors.blue(example));
            output.push("\n".to_string());
        }
        output.push("  ".to_string());
        output.push(desc.localize(locale, &[]));
        output.push("\n".to_string());
        output.push("\n".to_string());
    }

    let mut cli_commands: Vec<&OptionDecl> = Vec::new();
    let mut config_opts: Vec<&OptionDecl> = Vec::new();
    for opt in simple_options {
        if opt.is_command_line_only {
            cli_commands.push(opt);
        } else {
            config_opts.push(opt);
        }
    }

    output.extend(generate_section_options_output(
        sys,
        locale,
        &tsox_core::diagnostics::COMMAND_LINE_FLAGS.localize(locale, &[]),
        &cli_commands,
        false,
        None,
        None,
    ));
    let after = tsox_core::diagnostics::YOU_CAN_LEARN_ABOUT_ALL_OF_THE_COMPILER_OPTIONS_AT_0
        .localize(locale, &["https://aka.ms/tsc"]);
    output.extend(generate_section_options_output(
        sys,
        locale,
        &tsox_core::diagnostics::COMMON_COMPILER_OPTIONS.localize(locale, &[]),
        &config_opts,
        false,
        None,
        Some(&after),
    ));

    let mut writer = sys.writer();
    for chunk in &output {
        let _ = write!(writer, "{chunk}");
    }
}

pub fn print_all_help(sys: &dyn System, locale: &Locale, options: &[&OptionDecl]) {
    let mut output: Vec<String> = Vec::new();
    let msg = format!(
        "{} - {}",
        tsox_core::diagnostics::X_TSC_COLON_THE_TYPESCRIPT_COMPILER.localize(locale, &[]),
        tsox_core::diagnostics::VERSION_0.localize(locale, &[&tsox_core::core::mig::m3k::version()])
    );
    output.extend(get_header(sys, &create_colors(sys), &msg));

    let after_compiler_options = tsox_core::diagnostics::YOU_CAN_LEARN_ABOUT_ALL_OF_THE_COMPILER_OPTIONS_AT_0
        .localize(locale, &["https://aka.ms/tsc"]);
    output.extend(generate_section_options_output(
        sys, locale,
        &tsox_core::diagnostics::ALL_COMPILER_OPTIONS.localize(locale, &[]),
        options, true, None, Some(&after_compiler_options),
    ));

    let before_watch_options = tsox_core::diagnostics::INCLUDING_WATCH_W_WILL_START_WATCHING_THE_CURRENT_PROJECT_FOR_THE_FILE_CHANGES_ONCE_SET_YOU_CAN_CONFIG_WATCH_MODE_WITH_COLON.localize(locale, &[]);
    output.extend(generate_section_options_output(
        sys, locale,
        &tsox_core::diagnostics::WATCH_OPTIONS.localize(locale, &[]),
        &OPTIONS_FOR_WATCH.iter().collect::<Vec<_>>(),
        false, Some(&before_watch_options), None,
    ));

    let before_build_options = tsox_core::diagnostics::USING_BUILD_B_WILL_MAKE_TSC_BEHAVE_MORE_LIKE_A_BUILD_ORCHESTRATOR_THAN_A_COMPILER_THIS_IS_USED_TO_TRIGGER_BUILDING_COMPOSITE_PROJECTS_WHICH_YOU_CAN_LEARN_MORE_ABOUT_AT_0.localize(locale, &["https://aka.ms/tsc-composite-builds"]);
    let build_options: Vec<&OptionDecl> = BUILD_OPTIONS.iter().collect();
    output.extend(generate_section_options_output(
        sys, locale,
        &tsox_core::diagnostics::BUILD_OPTIONS.localize(locale, &[]),
        &build_options,
        false, Some(&before_build_options), None,
    ));

    let mut writer = sys.writer();
    for chunk in &output {
        let _ = write!(writer, "{chunk}");
    }
}

pub fn print_build_help(sys: &dyn System, locale: &Locale, build_options: &[&OptionDecl]) {
    let mut output: Vec<String> = Vec::new();
    let msg = format!(
        "{} - {}",
        tsox_core::diagnostics::X_TSC_COLON_THE_TYPESCRIPT_COMPILER.localize(locale, &[]),
        tsox_core::diagnostics::VERSION_0.localize(locale, &[&tsox_core::core::mig::m3k::version()])
    );
    output.extend(get_header(sys, &create_colors(sys), &msg));
    let before = tsox_core::diagnostics::USING_BUILD_B_WILL_MAKE_TSC_BEHAVE_MORE_LIKE_A_BUILD_ORCHESTRATOR_THAN_A_COMPILER_THIS_IS_USED_TO_TRIGGER_BUILDING_COMPOSITE_PROJECTS_WHICH_YOU_CAN_LEARN_MORE_ABOUT_AT_0.localize(locale, &["https://aka.ms/tsc-composite-builds"]);
    output.extend(generate_section_options_output(
        sys, locale,
        &tsox_core::diagnostics::BUILD_OPTIONS.localize(locale, &[]),
        build_options,
        false, Some(&before), None,
    ));

    let mut writer = sys.writer();
    for chunk in &output {
        let _ = write!(writer, "{chunk}");
    }
}

pub fn generate_section_options_output(
    sys: &dyn System,
    locale: &Locale,
    section_name: &str,
    options: &[&OptionDecl],
    sub_category: bool,
    before_options_description: Option<&str>,
    after_options_description: Option<&str>,
) -> Vec<String> {
    let mut output = vec![
        create_colors(sys).bold(section_name),
        "\n".to_string(),
        "\n".to_string(),
    ];
    if let Some(before) = before_options_description {
        output.push(before.to_string());
        output.push("\n".to_string());
        output.push("\n".to_string());
    }
    if !sub_category {
        output.extend(generate_group_option_output(sys, locale, options));
        if let Some(after) = after_options_description {
            output.push(after.to_string());
            output.push("\n".to_string());
            output.push("\n".to_string());
        }
        return output;
    }
    output.extend(generate_group_option_output(sys, locale, options));
    if let Some(after) = after_options_description {
        output.push(after.to_string());
        output.push("\n".to_string());
        output.push("\n".to_string());
    }
    output
}

pub fn generate_group_option_output(
    sys: &dyn System,
    locale: &Locale,
    options_list: &[&OptionDecl],
) -> Vec<String> {
    let max_length = options_list
        .iter()
        .map(|o| get_display_name_text_of_option(o).len())
        .max()
        .unwrap_or(0);

    let right_align_of_left_part = max_length + 2;
    let left_align_of_right_part = right_align_of_left_part + 2;

    let mut lines: Vec<String> = Vec::new();
    for option in options_list {
        lines.extend(generate_option_output(
            sys,
            locale,
            option,
            right_align_of_left_part,
            left_align_of_right_part,
        ));
    }

    if lines.len() < 2 || lines[lines.len() - 2] != "\n" {
        lines.push("\n".to_string());
    }
    lines
}
