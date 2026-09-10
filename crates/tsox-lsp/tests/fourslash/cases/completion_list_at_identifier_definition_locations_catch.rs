use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_at_identifier_definition_locations_catch() {
    let content = r#"var aa = 1;
 try {} catch(/*catchVariable1*/
 try {} catch(a/*catchVariable2*/"#;
    let mut s = Session::new_for_test("completionListAtIdentifierDefinitionLocations_catch", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), nil)
}
