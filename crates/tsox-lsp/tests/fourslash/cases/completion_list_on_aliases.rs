use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_on_aliases() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"namespace M {
    export var value;

    import x = M;
    /*1*/
    x./*2*/
}"#;
    let mut s = Session::new_for_test("completionListOnAliases", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["value"]);
}
