#![allow(dead_code, unused_imports, unused_variables)]

use crate::ast::diagnostic::Diagnostic;
use crate::ast::node_line_map::LineMap;
use crate::ast::SourceFile;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics::{Category, Message};
use tsox_core::locale::Locale;

pub static THIS_LOCATION_IS_IN_VIRTUAL_CODE_PRODUCED_BY_THE_CONTENT_MAPPER_0_AND_HAS_NO_CORRESPONDING_LOCATION_IN_THE_ORIGINAL_FILE: Message = Message {
    code: 100030,
    category: Category::Message,
    key: "This_location_is_in_virtual_code_produced_by_the_content_mapper_0_and_has_no_corresponding_location__100030",
    text: "This location is in virtual code produced by the content mapper '{0}' and has no corresponding location in the original file.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub const FOREGROUND_COLOR_ESCAPE_GREY: &str = "\u{1b}[90m";
pub const FOREGROUND_COLOR_ESCAPE_RED: &str = "\u{1b}[91m";
pub const FOREGROUND_COLOR_ESCAPE_YELLOW: &str = "\u{1b}[93m";
pub const FOREGROUND_COLOR_ESCAPE_BLUE: &str = "\u{1b}[94m";
pub const FOREGROUND_COLOR_ESCAPE_CYAN: &str = "\u{1b}[96m";
pub const GUTTER_STYLE_SEQUENCE: &str = "\u{1b}[7m";
pub const GUTTER_SEPARATOR: &str = " ";
pub const RESET_ESCAPE_SEQUENCE: &str = "\u{1b}[0m";
pub const ELLIPSIS: &str = "...";

#[derive(Clone, Debug, Default)]
pub struct FormattingOptions {
    pub locale: Locale,
    pub compare_paths_options: tsox_core::tspath::ComparePathsOptions,
    pub new_line: String,
}

pub trait FileLike {
    fn file_name(&self) -> &str;
    fn text(&self) -> &str;
    fn ecma_line_map(&self) -> &LineMap;
}

impl FileLike for SourceFile {
    fn file_name(&self) -> &str {
        &self.file_name
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn ecma_line_map(&self) -> &LineMap {
        &self.line_map
    }
}

pub struct OriginalTextFile {
    file_name: String,
    text: String,
    line_map: LineMap,
}

pub fn new_original_text_file(file: &SourceFile, file_name: String) -> OriginalTextFile {
    let text = file.text.clone();
    OriginalTextFile {
        file_name,
        line_map: LineMap::from_text(&text),
        text,
    }
}

impl FileLike for OriginalTextFile {
    fn file_name(&self) -> &str {
        &self.file_name
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn ecma_line_map(&self) -> &LineMap {
        &self.line_map
    }
}

pub struct RenamedFile {
    file: std::sync::Arc<SourceFile>,
    file_name: String,
}

impl FileLike for RenamedFile {
    fn file_name(&self) -> &str {
        &self.file_name
    }

    fn text(&self) -> &str {
        &self.file.text
    }

    fn ecma_line_map(&self) -> &LineMap {
        &self.file.line_map
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedLocation {
    pub loc: TextRange,
    pub use_original: bool,
    pub synthesized: bool,
}

pub trait AstDiagnosticExt {
    fn resolve(&self) -> ResolvedLocation;
    fn source(&self) -> &str;
    fn ast_pos(&self) -> usize;
    fn ast_end(&self) -> usize;
    fn ast_len(&self) -> usize;
    fn file_like(&self) -> Option<std::sync::Arc<dyn FileLike>>;
    fn wrapped_message_chain(&self) -> Vec<Diagnostic>;
    fn wrapped_related_information(&self) -> Vec<Diagnostic>;
}

impl AstDiagnosticExt for Diagnostic {
    fn resolve(&self) -> ResolvedLocation {
        let loc = self.loc;
        match &self.file {
            None => ResolvedLocation {
                loc,
                use_original: false,
                synthesized: false,
            },
            Some(_file) => {
                if !self.source().is_empty() {
                    return ResolvedLocation {
                        loc,
                        use_original: true,
                        synthesized: false,
                    };
                }
                ResolvedLocation {
                    loc,
                    use_original: false,
                    synthesized: false,
                }
            }
        }
    }

    fn source(&self) -> &str {
        ""
    }

    fn ast_pos(&self) -> usize {
        self.resolve().loc.pos() as usize
    }

    fn ast_end(&self) -> usize {
        self.resolve().loc.end() as usize
    }

    fn ast_len(&self) -> usize {
        self.resolve().loc.len() as usize
    }

    fn file_like(&self) -> Option<std::sync::Arc<dyn FileLike>> {
        let file = self.file.clone()?;
        let file_name = file.file_name.clone();
        if self.resolve().use_original {
            return Some(std::sync::Arc::new(new_original_text_file(
                &file,
                file_name,
            )));
        }
        if file_name != file.file_name {
            return Some(std::sync::Arc::new(RenamedFile {
                file,
                file_name,
            }));
        }
        Some(std::sync::Arc::new(clone_source_file_alias(&file)))
    }

    fn wrapped_message_chain(&self) -> Vec<Diagnostic> {
        let mut result: Vec<Diagnostic> = self
            .message_chain
            .iter()
            .map(wrap_ast_diagnostic_owned)
            .collect();
        if self.resolve().synthesized {
            let note = Diagnostic::new(
                None,
                TextRange::default(),
                THIS_LOCATION_IS_IN_VIRTUAL_CODE_PRODUCED_BY_THE_CONTENT_MAPPER_0_AND_HAS_NO_CORRESPONDING_LOCATION_IN_THE_ORIGINAL_FILE,
                vec![String::new()],
            );
            result.push(wrap_ast_diagnostic(note));
        }
        result
    }

    fn wrapped_related_information(&self) -> Vec<Diagnostic> {
        self.related_information
            .iter()
            .map(wrap_ast_diagnostic_owned)
            .collect()
    }
}

fn clone_source_file_alias(file: &SourceFile) -> SourceFileClone {
    SourceFileClone {
        file_name: file.file_name.clone(),
        text: file.text.clone(),
        line_map: LineMap::from_text(&file.text),
    }
}

pub struct SourceFileClone {
    file_name: String,
    text: String,
    line_map: LineMap,
}

impl FileLike for SourceFileClone {
    fn file_name(&self) -> &str {
        &self.file_name
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn ecma_line_map(&self) -> &LineMap {
        &self.line_map
    }
}

pub fn wrap_ast_diagnostic(d: Diagnostic) -> Diagnostic {
    d
}

pub fn wrap_ast_diagnostic_owned(d: &Diagnostic) -> Diagnostic {
    d.clone()
}

pub fn wrap_ast_diagnostics(diags: &[Diagnostic]) -> Vec<Diagnostic> {
    diags.iter().map(wrap_ast_diagnostic_owned).collect()
}

pub fn from_ast_diagnostics(diags: &[Diagnostic]) -> Vec<Diagnostic> {
    diags.iter().map(wrap_ast_diagnostic_owned).collect()
}

pub fn to_diagnostics(diags: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diags
}

pub fn diagnostic_prefix(diagnostic: &Diagnostic) -> String {
    let source = diagnostic.source();
    if !source.is_empty() {
        return source.to_string();
    }
    "TS".to_string()
}

pub fn get_category_format(category: Category) -> &'static str {
    match category {
        Category::Error => FOREGROUND_COLOR_ESCAPE_RED,
        Category::Warning => FOREGROUND_COLOR_ESCAPE_YELLOW,
        Category::Suggestion => FOREGROUND_COLOR_ESCAPE_GREY,
        Category::Message => FOREGROUND_COLOR_ESCAPE_BLUE,
    }
}

pub type FormattedWriter = fn(&mut dyn std::io::Write, &str, &str);

pub fn write_with_style_and_reset(output: &mut dyn std::io::Write, text: &str, format_style: &str) {
    let _ = std::io::Write::write_all(output, format_style.as_bytes());
    let _ = std::io::Write::write_all(output, text.as_bytes());
    let _ = std::io::Write::write_all(output, RESET_ESCAPE_SEQUENCE.as_bytes());
}

pub fn write_location(
    output: &mut dyn std::io::Write,
    file: &dyn FileLike,
    pos: usize,
    format_opts: &FormattingOptions,
    write_with_style: FormattedWriter,
) {
    let (first_line, first_char) =
        super::super::line_and_character(file.ecma_line_map(), file.text(), pos);
    let relative_file_name = tsox_core::tspath::convert_to_relative_path(
        file.file_name(),
        &format_opts.compare_paths_options,
    );

    write_with_style(output, &relative_file_name, FOREGROUND_COLOR_ESCAPE_CYAN);
    let _ = std::io::Write::write_all(output, b":");
    write_with_style(
        output,
        &(first_line + 1).to_string(),
        FOREGROUND_COLOR_ESCAPE_YELLOW,
    );
    let _ = std::io::Write::write_all(output, b":");
    write_with_style(
        output,
        &(first_char + 1).to_string(),
        FOREGROUND_COLOR_ESCAPE_YELLOW,
    );
}
