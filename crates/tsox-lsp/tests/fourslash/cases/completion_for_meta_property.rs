use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_meta_property() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"import./*1*/;
new./*2*/;
function test() { new./*3*/ }"#;
    let mut s = Session::new_for_test("completionForMetaProperty", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
