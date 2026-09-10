use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_at_identifier_definition_locations_var_declarations() {
    let content = r#"var aa = 1;
var /*varName1*/
var a/*varName2*/
var a2,/*varName3*/
var a2, a/*varName4*/"#;
    let mut s = Session::new_for_test("completionListAtIdentifierDefinitionLocations_varDeclarations", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), nil)
}
