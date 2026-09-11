use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_remove_unnecessary_await_not_available_on_return() {
    let content = r#"// @target: esnext
async function fn(): Promise<number> {
  return 0;
}"#;
    let mut s = Session::new_for_test("codeFixRemoveUnnecessaryAwait_notAvailableOnReturn", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}
