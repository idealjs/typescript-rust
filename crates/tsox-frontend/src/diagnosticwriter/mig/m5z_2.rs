#![allow(dead_code, unused_imports, unused_variables)]

use super::m5z::*;
use crate::ast::diagnostic::Diagnostic;
use crate::ast::SourceFile;
use crate::ast::utf16_len;
use tsox_core::diagnostics::Category;
use tsox_core::locale::Locale;

pub fn format_diagnostics_with_color_and_context(
    output: &mut dyn std::io::Write,
    diags: &[Diagnostic],
    format_opts: &FormattingOptions,
) {
    if diags.is_empty() {
        return;
    }
    for (i, diagnostic) in diags.iter().enumerate() {
        if i > 0 {
            let _ = std::io::Write::write_all(output, format_opts.new_line.as_bytes());
        }
        format_diagnostic_with_color_and_context(output, diagnostic, format_opts);
    }
}

pub fn format_diagnostic_with_color_and_context(
    output: &mut dyn std::io::Write,
    diagnostic: &Diagnostic,
    format_opts: &FormattingOptions,
) {
    if let Some(file) = diagnostic.file_like() {
        let pos = diagnostic.ast_pos();
        write_location(output, file.as_ref(), pos, format_opts, write_with_style_and_reset);
        let _ = std::io::Write::write_all(output, b" - ");
    }

    write_with_style_and_reset(
        output,
        diagnostic.category.name(),
        get_category_format(diagnostic.category),
    );
    let _ = write!(
        output,
        "{} {}{}: {}",
        FOREGROUND_COLOR_ESCAPE_GREY,
        diagnostic_prefix(diagnostic),
        diagnostic.code,
        RESET_ESCAPE_SEQUENCE
    );
    write_flattened_diagnostic_message(output, diagnostic, &format_opts.new_line, &format_opts.locale);

    if diagnostic.file.is_some()
        && diagnostic.code != tsox_core::diagnostics::FILE_APPEARS_TO_BE_BINARY.code
    {
        let _ = std::io::Write::write_all(output, format_opts.new_line.as_bytes());
        write_code_snippet(
            output,
            diagnostic.file_like().as_deref().unwrap(),
            diagnostic.ast_pos(),
            diagnostic.ast_len(),
            get_category_format(diagnostic.category),
            "",
            format_opts,
        );
        let _ = std::io::Write::write_all(output, format_opts.new_line.as_bytes());
    }

    let related = diagnostic.wrapped_related_information();
    if !related.is_empty() {
        for related_information in &related {
            if let Some(file) = related_information.file_like() {
                let _ = std::io::Write::write_all(output, format_opts.new_line.as_bytes());
                let _ = std::io::Write::write_all(output, b"  ");
                let pos = related_information.ast_pos();
                write_location(
                    output,
                    file.as_ref(),
                    pos,
                    format_opts,
                    write_with_style_and_reset,
                );
                let _ = std::io::Write::write_all(output, b" - ");
                write_flattened_diagnostic_message(
                    output,
                    related_information,
                    &format_opts.new_line,
                    &format_opts.locale,
                );
                write_code_snippet(
                    output,
                    file.as_ref(),
                    pos,
                    related_information.ast_len(),
                    FOREGROUND_COLOR_ESCAPE_CYAN,
                    "    ",
                    format_opts,
                );
            }
            let _ = std::io::Write::write_all(output, format_opts.new_line.as_bytes());
        }
    }
}

pub fn write_code_snippet(
    writer: &mut dyn std::io::Write,
    source_file: &dyn FileLike,
    start: usize,
    length: usize,
    squiggle_color: &str,
    indent: &str,
    format_opts: &FormattingOptions,
) {
    let text = source_file.text();
    let line_map = source_file.ecma_line_map();
    let (first_line, first_line_char) = super::super::line_and_character(line_map, text, start);
    let (last_line, mut last_line_char) =
        super::super::line_and_character(line_map, text, start + length);
    if length == 0 {
        last_line_char += 1;
    }

    let last_line_of_file = line_map.line_at(text.len());

    let has_more_than_five_lines = last_line.saturating_sub(first_line) >= 4;
    let mut gutter_width = (last_line + 1).to_string().len();
    if has_more_than_five_lines {
        gutter_width = gutter_width.max(ELLIPSIS.len());
    }

    let mut i = first_line;
    while i <= last_line {
        let _ = std::io::Write::write_all(writer, format_opts.new_line.as_bytes());

        if has_more_than_five_lines && first_line + 1 < i && i < last_line.saturating_sub(1) {
            let _ = std::io::Write::write_all(writer, indent.as_bytes());
            let _ = std::io::Write::write_all(writer, GUTTER_STYLE_SEQUENCE.as_bytes());
            let _ = write!(writer, "{:>width$}", ELLIPSIS, width = gutter_width);
            let _ = std::io::Write::write_all(writer, RESET_ESCAPE_SEQUENCE.as_bytes());
            let _ = std::io::Write::write_all(writer, GUTTER_SEPARATOR.as_bytes());
            let _ = std::io::Write::write_all(writer, format_opts.new_line.as_bytes());
            i = last_line.saturating_sub(1);
        }

        let line_start = line_map.line_starts[i] as usize;
        let line_end = if i < last_line_of_file {
            line_map.line_starts[i + 1] as usize
        } else {
            text.len()
        };

        let line_content = text[line_start..line_end]
            .trim_end_matches(char::is_whitespace)
            .replace('\t', " ");

        let _ = std::io::Write::write_all(writer, indent.as_bytes());
        let _ = std::io::Write::write_all(writer, GUTTER_STYLE_SEQUENCE.as_bytes());
        let _ = write!(writer, "{:>width$}", i + 1, width = gutter_width);
        let _ = std::io::Write::write_all(writer, RESET_ESCAPE_SEQUENCE.as_bytes());
        let _ = std::io::Write::write_all(writer, GUTTER_SEPARATOR.as_bytes());
        let _ = std::io::Write::write_all(writer, line_content.as_bytes());
        let _ = std::io::Write::write_all(writer, format_opts.new_line.as_bytes());

        let _ = std::io::Write::write_all(writer, indent.as_bytes());
        let _ = std::io::Write::write_all(writer, GUTTER_STYLE_SEQUENCE.as_bytes());
        let _ = write!(writer, "{:>width$}", "", width = gutter_width);
        let _ = std::io::Write::write_all(writer, RESET_ESCAPE_SEQUENCE.as_bytes());
        let _ = std::io::Write::write_all(writer, GUTTER_SEPARATOR.as_bytes());
        let _ = std::io::Write::write_all(writer, squiggle_color.as_bytes());
        if i == first_line {
            let last_char_for_line = if i == last_line {
                last_line_char
            } else {
                utf16_len(&line_content)
            };

            let _ = std::io::Write::write_all(writer, " ".repeat(first_line_char).as_bytes());
            let _ = std::io::Write::write_all(
                writer,
                "~".repeat(last_char_for_line.saturating_sub(first_line_char))
                    .as_bytes(),
            );
        } else if i == last_line {
            let _ = std::io::Write::write_all(writer, "~".repeat(last_line_char).as_bytes());
        } else {
            let _ = std::io::Write::write_all(
                writer,
                "~".repeat(utf16_len(&line_content)).as_bytes(),
            );
        }

        let _ = std::io::Write::write_all(writer, RESET_ESCAPE_SEQUENCE.as_bytes());
        i += 1;
    }
}

pub fn flatten_diagnostic_message(
    d: &Diagnostic,
    new_line: &str,
    locale: &Locale,
) -> String {
    let mut output = Vec::new();
    write_flattened_diagnostic_message(&mut output, d, new_line, locale);
    String::from_utf8(output).unwrap_or_default()
}

pub fn write_flattened_ast_diagnostic_message(
    writer: &mut dyn std::io::Write,
    diagnostic: &Diagnostic,
    newline: &str,
    locale: &Locale,
) {
    write_flattened_diagnostic_message(writer, &wrap_ast_diagnostic_owned(diagnostic), newline, locale)
}

pub fn write_flattened_diagnostic_message(
    writer: &mut dyn std::io::Write,
    diagnostic: &Diagnostic,
    newline: &str,
    locale: &Locale,
) {
    let args: Vec<&str> = diagnostic.message_args.iter().map(|s| s.as_str()).collect();
    match &diagnostic.message {
        Some(msg) => {
            let _ = std::io::Write::write_all(writer, msg.localize(locale, &args).as_bytes());
        }
        None => {
            let _ = std::io::Write::write_all(
                writer,
                diagnostic.message_args.join("").as_bytes(),
            );
        }
    }

    for chain in diagnostic.wrapped_message_chain() {
        flatten_diagnostic_message_chain(writer, &chain, newline, locale, 1);
    }
}

pub fn flatten_diagnostic_message_chain(
    writer: &mut dyn std::io::Write,
    chain: &Diagnostic,
    new_line: &str,
    locale: &Locale,
    level: usize,
) {
    let _ = std::io::Write::write_all(writer, new_line.as_bytes());
    for _ in 0..level {
        let _ = std::io::Write::write_all(writer, b"  ");
    }

    let args: Vec<&str> = chain.message_args.iter().map(|s| s.as_str()).collect();
    match &chain.message {
        Some(msg) => {
            let _ = std::io::Write::write_all(writer, msg.localize(locale, &args).as_bytes());
        }
        None => {
            let _ = std::io::Write::write_all(writer, chain.message_args.join("").as_bytes());
        }
    }
    for child in chain.wrapped_message_chain() {
        flatten_diagnostic_message_chain(writer, &child, new_line, locale, level + 1);
    }
}

pub struct ErrorSummary {
    pub total_error_count: usize,
    pub global_errors: Vec<Diagnostic>,
    pub errors_by_file: Vec<(String, Vec<Diagnostic>)>,
    pub sorted_files: Vec<String>,
}

pub fn get_error_summary(diags: &[Diagnostic]) -> ErrorSummary {
    let mut total_error_count = 0usize;
    let mut global_errors = Vec::new();
    let mut errors_by_file: std::collections::BTreeMap<String, Vec<Diagnostic>> =
        std::collections::BTreeMap::new();

    for diagnostic in diags {
        if diagnostic.category != Category::Error {
            continue;
        }

        total_error_count += 1;
        match &diagnostic.file {
            None => global_errors.push(diagnostic.clone()),
            Some(file) => {
                errors_by_file
                    .entry(file.file_name.clone())
                    .or_default()
                    .push(diagnostic.clone());
            }
        }
    }

    let sorted_files: Vec<String> = errors_by_file.keys().cloned().collect();
    ErrorSummary {
        total_error_count,
        global_errors,
        errors_by_file: errors_by_file.into_iter().collect(),
        sorted_files,
    }
}

pub fn write_error_summary_text(
    output: &mut dyn std::io::Write,
    all_diagnostics: &[Diagnostic],
    format_opts: &FormattingOptions,
) {
    let error_summary = get_error_summary(all_diagnostics);
    let total_error_count = error_summary.total_error_count;
    if total_error_count == 0 {
        return;
    }

    let first_file = error_summary.sorted_files.first();
    let first_file_errors: Vec<Diagnostic> = first_file
        .and_then(|name| {
            error_summary
                .errors_by_file
                .iter()
                .find(|(f, _)| Some(f) == first_file)
                .map(|(_, errs)| errs.clone())
        })
        .unwrap_or_default();
    let first_file_name =
        pretty_path_for_file_error(first_file.map(String::as_str), &first_file_errors, format_opts);
    let num_erroring_files = error_summary.errors_by_file.len();

    let message;
    if total_error_count == 1 {
        if !error_summary.global_errors.is_empty() || first_file_name.is_empty() {
            message = tsox_core::diagnostics::FOUND_1_ERROR.localize(
                &format_opts.locale,
                &[],
            );
        } else {
            message = tsox_core::diagnostics::FOUND_1_ERROR_IN_0.localize(
                &format_opts.locale,
                &[&first_file_name],
            );
        }
    } else {
        match num_erroring_files {
            0 => {
                message = tsox_core::diagnostics::FOUND_0_ERRORS.localize(
                    &format_opts.locale,
                    &[&total_error_count.to_string()],
                );
            }
            1 => {
                message =
                    tsox_core::diagnostics::FOUND_0_ERRORS_IN_THE_SAME_FILE_STARTING_AT_COLON_1
                        .localize(
                            &format_opts.locale,
                            &[&total_error_count.to_string(), &first_file_name],
                        );
            }
            _ => {
                message = tsox_core::diagnostics::FOUND_0_ERRORS_IN_1_FILES.localize(
                    &format_opts.locale,
                    &[&total_error_count.to_string(), &num_erroring_files.to_string()],
                );
            }
        }
    }
    let _ = std::io::Write::write_all(output, format_opts.new_line.as_bytes());
    let _ = std::io::Write::write_all(output, message.as_bytes());
    let _ = std::io::Write::write_all(output, format_opts.new_line.as_bytes());
    let _ = std::io::Write::write_all(output, format_opts.new_line.as_bytes());
    if num_erroring_files > 1 {
        write_tabular_errors_display(output, &error_summary, format_opts);
        let _ = std::io::Write::write_all(output, format_opts.new_line.as_bytes());
    }
}

pub fn write_tabular_errors_display(
    output: &mut dyn std::io::Write,
    error_summary: &ErrorSummary,
    format_opts: &FormattingOptions,
) {
    let max_errors = error_summary
        .errors_by_file
        .iter()
        .map(|(_, errs)| errs.len())
        .max()
        .unwrap_or(0);

    let header_row = tsox_core::diagnostics::ERRORS_FILES.localize(&format_opts.locale, &[]);
    let left_column_heading_length = header_row.split(' ').next().map_or(0, |s| s.len());
    let length_of_biggest_error_count = max_errors.to_string().len();
    let left_padding_goal = left_column_heading_length.max(length_of_biggest_error_count);
    let header_padding =
        length_of_biggest_error_count.saturating_sub(left_column_heading_length);

    let _ = std::io::Write::write_all(output, " ".repeat(header_padding).as_bytes());
    let _ = std::io::Write::write_all(output, header_row.as_bytes());
    let _ = std::io::Write::write_all(output, format_opts.new_line.as_bytes());

    for file in &error_summary.sorted_files {
        let file_errors = error_summary
            .errors_by_file
            .iter()
            .find(|(f, _)| f == file)
            .map(|(_, errs)| errs.as_slice())
            .unwrap_or(&[]);
        let error_count = file_errors.len();

        let _ = write!(output, "{:>width$}  ", error_count, width = left_padding_goal);
        let _ = std::io::Write::write_all(
            output,
            pretty_path_for_file_error(Some(file.as_str()), file_errors, format_opts).as_bytes(),
        );
        let _ = std::io::Write::write_all(output, format_opts.new_line.as_bytes());
    }
}

pub fn pretty_path_for_file_error(
    file: Option<&str>,
    file_errors: &[Diagnostic],
    format_opts: &FormattingOptions,
) -> String {
    let Some(file_name) = file else {
        return String::new();
    };
    if file_errors.is_empty() {
        return String::new();
    }
    let pos = file_errors[0].ast_pos();
    let line = {
        let source = file_errors[0].file.as_ref();
        source.map_or(0usize, |f| f.line_map.line_at(pos.min(f.text.len())))
    };
    let mut name = file_name.to_string();
    if tsox_core::tspath::path_is_absolute(&name)
        && tsox_core::tspath::path_is_absolute(&format_opts.compare_paths_options.current_directory)
    {
        name = tsox_core::tspath::convert_to_relative_path(
            file_name,
            &format_opts.compare_paths_options,
        );
    }
    format!(
        "{}{}:{}{}",
        name,
        FOREGROUND_COLOR_ESCAPE_GREY,
        line + 1,
        RESET_ESCAPE_SEQUENCE
    )
}

pub fn write_format_diagnostics(
    output: &mut dyn std::io::Write,
    diagnostics: &[Diagnostic],
    format_opts: &FormattingOptions,
) {
    for diagnostic in diagnostics {
        write_format_diagnostic(output, diagnostic, format_opts);
    }
}

pub fn write_format_diagnostic(
    output: &mut dyn std::io::Write,
    diagnostic: &Diagnostic,
    format_opts: &FormattingOptions,
) {
    if let Some(file) = &diagnostic.file {
        let (line, character) = super::super::line_and_character(
            &file.line_map,
            &file.text,
            diagnostic.ast_pos(),
        );
        let file_name = &file.file_name;
        let relative_file_name = tsox_core::tspath::convert_to_relative_path(
            file_name,
            &format_opts.compare_paths_options,
        );
        let _ = write!(output, "{}({},{}): ", relative_file_name, line + 1, character + 1);
    }

    let _ = write!(
        output,
        "{} {}{}: ",
        diagnostic.category.name(),
        diagnostic_prefix(diagnostic),
        diagnostic.code
    );
    write_flattened_diagnostic_message(
        output,
        diagnostic,
        &format_opts.new_line,
        &format_opts.locale,
    );
    let _ = std::io::Write::write_all(output, format_opts.new_line.as_bytes());
}

pub const SCREEN_STARTING_CODES: &[i32] = &[
    tsox_core::diagnostics::STARTING_COMPILATION_IN_WATCH_MODE.code,
    tsox_core::diagnostics::FILE_CHANGE_DETECTED_STARTING_INCREMENTAL_COMPILATION.code,
];

pub fn try_clear_screen(
    output: &mut dyn std::io::Write,
    diag: &Diagnostic,
    options: &tsox_core::core::compiler_options::CompilerOptions,
) -> bool {
    if !options.preserve_watch_output.is_true()
        && !options.extended_diagnostics.is_true()
        && !options.diagnostics.is_true()
        && SCREEN_STARTING_CODES.contains(&diag.code)
    {
        let _ = std::io::Write::write_all(output, b"\x1b[2J\x1b[3J\x1b[H");
        return true;
    }
    false
}
