use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn quick_info_js_doc_param_with_trailing_at_before_comment_end() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/** @param {string} x trailing @/*at*/*/
function /*fn*/foo(/*x*/x) {}
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "fn", "function foo(x: string): void", "\n\n*@param* `x` — trailing @")
    fourslash::verify_quick_info_at(&mut s, "x", "(parameter) x: string", "trailing @");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "at", &fourslash.CompletionsExpectedList{
}
