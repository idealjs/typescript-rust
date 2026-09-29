use tsox_lsp::fourslash::Session;


#[test]
fn completion_entry_for_const() {
    let content = r#"const c = "s";
/*1*/
const d = 1
d/*2*/
const e = 1
/*3*/"#;
    let _s = Session::new_for_test("completionEntryForConst", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"3"}, &fourslash.CompletionsExpectedList{
}
