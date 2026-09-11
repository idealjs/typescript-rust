use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_js_doc_param_with_trailing_at_before_comment_end() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/** @param {string} x trailing @/*at*/*/
function /*fn*/foo(/*x*/x) {}
"#;
    let mut s = Session::new_for_test("quickInfoJSDocParamWithTrailingAtBeforeCommentEnd", content);
    // TODO: f.VerifyQuickInfoAt(t, "fn", "function foo(x: string): void", "\n\n*@param* `x` — trailing @")
    fourslash::verify_quick_info_at(&mut s, "x", "(parameter) x: string", "trailing @");
    // TODO: f.VerifyCompletions(t, "at", &fourslash.CompletionsExpectedList{
}
