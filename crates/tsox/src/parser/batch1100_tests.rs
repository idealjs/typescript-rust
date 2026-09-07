use super::*;

#[test]
fn parse_constructor_param_static_modifier() {
    let (_, diags) = Parser::parse_source_file_text_with_diagnostics(
        "a.ts",
        "class foo {\n    constructor (static a: number) {\n    }\n}".to_string(),
    );
    eprintln!("diags: {diags:?}");
    let mut p = Parser::new("constructor (static a: number) {}");
    let _ = p.parse_expression();
}
