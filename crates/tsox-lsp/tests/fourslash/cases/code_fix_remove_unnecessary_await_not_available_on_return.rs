use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn code_fix_remove_unnecessary_await_not_available_on_return() {
    let content = r#"// @target: esnext
async function fn(): Promise<number> {
  return 0;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, nil)
}
