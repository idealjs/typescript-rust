use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_tuple() {
    let content = r#"declare const x: [number, number];
x[|./**/|];"#;
    let mut s = Session::new_for_test("completionsTuple", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
