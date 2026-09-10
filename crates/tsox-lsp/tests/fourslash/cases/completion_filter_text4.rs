use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_filter_text4() {
    let content = r#"declare const x: [number, number];
x[|.|]/**/;
"#;
    let mut s = Session::new_for_test("completionFilterText4", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
