use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_tuple() {
    let content = r#"declare const x: [number, number];
x[|./**/|];"#;
    let mut s = Session::new_for_test("completionsTuple", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
