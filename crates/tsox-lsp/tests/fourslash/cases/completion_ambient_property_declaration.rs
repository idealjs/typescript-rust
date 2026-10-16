use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_ambient_property_declaration() {
    let content = r#"class C {
    /*1*/ declare property: number;
    /*2*/
}"#;
    let mut s = Session::new_for_test("completionAmbientPropertyDeclaration", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
