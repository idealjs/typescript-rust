use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionJSDocNoCrash", content);
    // TODO: // The assertion here is simply "does not crash/panic".
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
