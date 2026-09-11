use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_js_doc_no_crash2() {
    let content = r#"
// @allowJs: true
// @filename: file.js
/**
 * @param {Object} obj
 * @param {string} obj.first The first property
 * @param {string} obj.second The second property
 * @param {string} obj.third The {@link foo} third property
 */
/*1*/function foo(obj) {}
"#;
    let mut s = Session::new_for_test("completionJSDocNoCrash2", content);
    // TODO: // The assertion here is simply "does not crash/panic".
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
