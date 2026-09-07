use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn triple_slash_ref_path_completion_context() {
    let content = r#"// @Filename: f.ts
/*f*/
// @Filename: test.ts
/// <reference path/*0*/=/*1*/"/*8*/
/// <reference path/*2*/=/*3*/"/*9*/"/*4*/ /*5*///*6*/>/*7*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"0", "1", "2", "3", "4", "5", "6", "7"}, nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"8", "9"}, &fourslash.CompletionsExpectedList{
}
