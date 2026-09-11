use tsox_lsp::fourslash::{self, Session};


#[test]
fn unreachable_code_diagnostics() {
    let content = r#"// @allowUnreachableCode: false
throw new Error();
	
(() => {})();
	"#;
    let mut s = Session::new_for_test("unreachableCodeDiagnostics", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
