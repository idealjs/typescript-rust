use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_after_newline() {
    let content = r#"// @lib: es5
let foo /*1*/
/*2*/
/*3*/"#;
    let mut s = Session::new_for_test("completionAfterNewline", content);
    fourslash::verify_completions_empty_at(&mut s, Some("1"));
    // TODO: f.VerifyCompletions(t, []string{"2", "3"}, &fourslash.CompletionsExpectedList{
}
