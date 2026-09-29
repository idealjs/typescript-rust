use super::m5z_2::write_flattened_diagnostic_message;
use super::m5z::{write_with_style_and_reset, FOREGROUND_COLOR_ESCAPE_GREY};
use crate::ast::diagnostic::Diagnostic;
use super::m5z::FormattingOptions;

pub fn format_diagnostics_status_and_time(
    writer: &mut dyn std::io::Write,
    time: &str,
    diagnostic: &Diagnostic,
    format_opts: &FormattingOptions,
) {
    let _ = write!(writer, "{} - ", time);
    write_flattened_diagnostic_message(
        writer,
        diagnostic,
        &format_opts.new_line,
        &format_opts.locale,
    );
}

pub fn format_diagnostics_status_with_color_and_time(
    writer: &mut dyn std::io::Write,
    time: &str,
    diagnostic: &Diagnostic,
    format_opts: &FormattingOptions,
) {
    let _ = write!(writer, "[");
    write_with_style_and_reset(writer, time, FOREGROUND_COLOR_ESCAPE_GREY);
    let _ = write!(writer, "] ");
    write_flattened_diagnostic_message(
        writer,
        diagnostic,
        &format_opts.new_line,
        &format_opts.locale,
    );
}
