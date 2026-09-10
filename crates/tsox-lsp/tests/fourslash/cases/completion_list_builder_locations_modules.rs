use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_builder_locations_modules() {
    let content = r#"// @lib: es5
module A/*moduleName1*/
module A./*moduleName2*/"#;
    let mut s = Session::new_for_test("completionListBuilderLocations_Modules", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "moduleName1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "moduleName2", &fourslash.CompletionsExpectedList{
}
