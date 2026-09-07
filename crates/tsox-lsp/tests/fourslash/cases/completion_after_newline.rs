use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_after_newline() {
    let content = r#"// @lib: es5
let foo /*1*/
/*2*/
/*3*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"2", "3"}, &fourslash.CompletionsExpectedList{
}
