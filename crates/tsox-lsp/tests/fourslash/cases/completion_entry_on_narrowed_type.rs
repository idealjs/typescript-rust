use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
