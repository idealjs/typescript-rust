use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_entry_on_narrowed_type() {
    let content = r#"function foo(strOrNum: string | number) {
    /*1*/
    if (typeof strOrNum === "number") {
        strOrNum/*2*/;
    }
    else {
        strOrNum/*3*/;
    }
}"#;
    let mut s = Session::new_for_test("completionEntryOnNarrowedType", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
