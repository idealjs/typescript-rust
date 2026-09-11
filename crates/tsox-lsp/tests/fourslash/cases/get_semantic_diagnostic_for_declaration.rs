use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_semantic_diagnostic_for_declaration() {
    let content = r#"// @strict: false
// @module: CommonJS
// @declaration: true
export function /*1*/foo/*2*/() {
    interface privateInterface {}
    class Bar implements privateInterface { private a; }
    return Bar;
}"#;
    let mut s = Session::new_for_test("getSemanticDiagnosticForDeclaration", content);
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
