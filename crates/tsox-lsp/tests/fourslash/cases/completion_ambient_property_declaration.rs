use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_ambient_property_declaration() {
    let content = r#"class C {
    /*1*/ declare property: number;
    /*2*/
}"#;
    let mut s = Session::new_for_test("completionAmbientPropertyDeclaration", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
