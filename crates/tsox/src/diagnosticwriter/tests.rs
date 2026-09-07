use super::*;
use crate::ast::SourceFile;
use crate::core::text::TextRange;
use crate::diagnostics::new_ad_hoc_message;

fn make_file(text: &str) -> Arc<SourceFile> {
    use crate::ast::{Node, NodeData, NodeList, SyntaxKind};
    let line_map = LineMap::from_text(text);
    Arc::new(SourceFile {
        node: Arc::new(Node::with_loc(
            SyntaxKind::SourceFile,
            NodeData::SourceFile(crate::ast::node_data_generated::SourceFileData {
                statements: Arc::new(NodeList::default()),
                end_of_file_token: Arc::new(Node::with_loc(
                    SyntaxKind::EndOfFile,
                    NodeData::Token,
                    TextRange::new(text.len(), text.len()),
                )),
            }),
            TextRange::new(0, text.len()),
        )),
        file_name: "test.ts".to_string(),
        text: text.to_string(),
        line_map,
        language_variant: crate::ast::LanguageVariant::Standard,
        script_kind: crate::ast::ScriptKind::Ts,
        comment_directives: Vec::new(),
        jsdoc_cache: std::sync::RwLock::new(std::collections::HashMap::new()),
        has_lazy_jsdoc: true,
        is_declaration_file: false,
        imports: Vec::new(),
        module_augmentations: Vec::new(),
        ambient_module_names: Vec::new(),
        parse_error_spans: Vec::new(),
        external_module_indicator: None,
        common_js_module_indicator: None,
        uses_uri_style_node_core_modules: crate::core::tristate::Tristate::Unknown,
        has_parse_diagnostics: false,
    })
}

#[test]
fn line_and_character_basic() {
    let file = make_file("abc\ndef\nghi");
    let (line, col) = line_and_character(&file.line_map, &file.text, 5);
    assert_eq!(line, 1);
    assert_eq!(col, 1);
}

#[test]
fn compact_format() {
    let file = make_file("abc\ndef");
    let diag = Diagnostic::new(
        Some(file),
        TextRange::new(5, 6),
        new_ad_hoc_message("Cannot find name 'x'."),
        vec![],
    );
    let s = format_diagnostic_compact(&diag, None);
    assert_eq!(s, "test.ts(2,2): error TS-1: Cannot find name 'x'.");
}

#[test]
fn pretty_format_has_squiggle() {
    let file = make_file("let x = 1");
    let diag = Diagnostic::new(
        Some(file),
        TextRange::new(4, 5),
        new_ad_hoc_message("oops"),
        vec![],
    );
    let s = format_diagnostic_pretty(&diag, None);
    assert!(s.contains("test.ts:1:5 - error TS-1: oops"));
    assert!(s.contains("~"));
}
