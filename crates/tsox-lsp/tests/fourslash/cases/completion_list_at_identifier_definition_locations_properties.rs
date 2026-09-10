use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_at_identifier_definition_locations_properties() {
    let content = r#"var aa = 1;
class A1 {
    /*property1*/
}
class A2 {
    p/*property2*/
}
class A3 {
    public s/*property3*/
}
class A4 {
    a/*property4*/
}
class A5 {
    public a/*property5*/
}
class A6 {
    protected a/*property6*/
}
class A7 {
    private a/*property7*/
}"#;
    let mut s = Session::new_for_test("completionListAtIdentifierDefinitionLocations_properties", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
