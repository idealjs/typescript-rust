use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // The assertion here is simply 'does not crash/panic'."]
#[test]
fn completion_js_doc_no_crash() {
    let content = r#"
// @allowJs: true
// @filename: file.js
class ErrorMap {
  /**
   * @type {string}
   *//*1*/
  errorMap;
}
"#;
    let mut s = Session::new(content);
    // TODO: // The assertion here is simply "does not crash/panic".
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
