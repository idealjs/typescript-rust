use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics02() {
    let content = r#"// @lib: es5
// @allowJs: true
// @Filename: b.js
var a = "a";
var b: boolean = true;
function foo(): string { }
var var = "c";"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics02", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
