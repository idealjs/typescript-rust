use tsox_lsp::fourslash::Session;


#[test]
fn unreachable_code_diagnostics() {
    let content = r#"// @allowUnreachableCode: false
throw new Error();
	
(() => {})();
	"#;
    let _s = Session::new_for_test("unreachableCodeDiagnostics", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
