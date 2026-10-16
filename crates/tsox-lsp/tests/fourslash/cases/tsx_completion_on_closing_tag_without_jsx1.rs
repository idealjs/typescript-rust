use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_completion_on_closing_tag_without_jsx1() {
    let content = r#"//@Filename: file.tsx
var x1 = <div><//**/"#;
    let mut s = Session::new_for_test("tsxCompletionOnClosingTagWithoutJSX1", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["div>"]);
}
