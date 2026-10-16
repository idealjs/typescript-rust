use tsox_lsp::fourslash::Session;


#[test]
fn completion_list_at_identifier_definition_locations_catch() {
    let content = r#"var aa = 1;
 try {} catch(/*catchVariable1*/
 try {} catch(a/*catchVariable2*/"#;
    let _s = Session::new_for_test("completionListAtIdentifierDefinitionLocations_catch", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), nil)
}
