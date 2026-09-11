use tsox_lsp::fourslash::{self, Session};


#[test]
fn const_quick_info_and_completion_list() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"const /*1*/a = 10;
var x = /*2*/a;
/*3*/
function foo() {
    const /*4*/b = 20;
    var y = /*5*/b;
    var z = /*6*/a;
    /*7*/
}"#;
    let mut s = Session::new_for_test("constQuickInfoAndCompletionList", content);
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"5", "6"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
    fourslash::verify_quick_info_at(&mut s, "1", "const a: 10", "");
    fourslash::verify_quick_info_at(&mut s, "2", "const a: 10", "");
    fourslash::verify_quick_info_at(&mut s, "4", "const b: 20", "");
    fourslash::verify_quick_info_at(&mut s, "5", "const b: 20", "");
    fourslash::verify_quick_info_at(&mut s, "6", "const a: 10", "");
}
