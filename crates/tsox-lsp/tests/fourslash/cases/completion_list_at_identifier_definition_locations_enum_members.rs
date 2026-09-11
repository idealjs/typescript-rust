use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_identifier_definition_locations_enum_members() {
    let content = r#"var aa = 1;
enum a { /*enumValueName1*/"#;
    let mut s = Session::new_for_test("completionListAtIdentifierDefinitionLocations_enumMembers", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), nil)
    // TODO: }
}
