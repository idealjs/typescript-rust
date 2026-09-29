use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_with_unterminated_js_doc_ending_with_at1() {
    let content = r#"// @allowJs: true
// @Filename: /atOnNewLineAtEOF.js
function foo(x) {}
/**
 * @/*1*/"#;
    let mut s = Session::new_for_test("completionWithUnterminatedJSDocEndingWithAt1", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
