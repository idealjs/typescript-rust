use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn get_java_script_syntactic_diagnostics02() {
    let content = r#"// @lib: es5
// @allowJs: true
// @Filename: b.js
var a = "a";
var b: boolean = true;
function foo(): string { }
var var = "c";"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineNonSuggestionDiagnostics"); // f.VerifyBaselineNonSuggestionDiagnostics(t)
}
