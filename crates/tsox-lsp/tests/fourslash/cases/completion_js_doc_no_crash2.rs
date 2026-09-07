use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // The assertion here is simply 'does not crash/panic'."]
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
    let mut s = Session::new(content);
    // TODO: // The assertion here is simply "does not crash/panic".
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
