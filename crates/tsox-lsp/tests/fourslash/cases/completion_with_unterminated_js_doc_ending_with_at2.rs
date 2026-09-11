use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_with_unterminated_js_doc_ending_with_at2() {
    let content = r#"// @allowJs: true
// @Filename: /atInTextAtEOF.js
function foo(x) {}
/** some text @/*1*/"#;
    let mut s = Session::new_for_test("completionWithUnterminatedJSDocEndingWithAt2", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
