use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_builder_locations_properties() {
    let content = r#"var aa = 1;
class A1 {
    public static /*property1*/
}
class A2 {
    public static a/*property2*/
}"#;
    let mut s = Session::new_for_test("completionListBuilderLocations_properties", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
