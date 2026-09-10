use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_entry_for_const() {
    let content = r#"const c = "s";
/*1*/
const d = 1
d/*2*/
const e = 1
/*3*/"#;
    let mut s = Session::new_for_test("completionEntryForConst", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"3"}, &fourslash.CompletionsExpectedList{
}
