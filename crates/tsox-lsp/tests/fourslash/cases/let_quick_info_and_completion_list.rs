use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn let_quick_info_and_completion_list() {
    let content = r#"let /*1*/a = 10;
/*2*/a = 30;
function foo() {
    let /*3*/b = 20;
    /*4*/b = /*5*/a;
}"#;
    let mut s = Session::new_for_test("letQuickInfoAndCompletionList", content);
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "5");
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::verify_quick_info_at(&mut s, "1", "let a: number", "");
    fourslash::verify_quick_info_at(&mut s, "2", "let a: number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "let b: number", "");
    fourslash::verify_quick_info_at(&mut s, "4", "let b: number", "");
    fourslash::verify_quick_info_at(&mut s, "5", "let a: number", "");
}
