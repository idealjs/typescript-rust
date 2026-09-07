use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_after_trailing_at_in_js_doc1() {
    let content = r#"// @allowJs: true
// @Filename: /atTagPosition.js
/**
 * @/*1*/
 */
function foo(x) {}

// @Filename: /atAfterExistingParam.js
/**
 * @param {string} x ok
 * @/*2*/
 */
function bar(x, y) {}

// @Filename: /atMidLine.js
/**
 * some text @/*3*/
 */
function baz(y) {}
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2", "3"}, &fourslash.CompletionsExpectedList{
}
