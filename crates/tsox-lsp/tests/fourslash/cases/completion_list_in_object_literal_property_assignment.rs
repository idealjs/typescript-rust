use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_literal_property_assignment() {
    let content = r#"var foo;
interface I {
    metadata: string;
    wat: string;
}
var x: I = {
    metadata: "/*1*/
}"#;
    let mut s = Session::new_for_test("completionListInObjectLiteralPropertyAssignment", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
