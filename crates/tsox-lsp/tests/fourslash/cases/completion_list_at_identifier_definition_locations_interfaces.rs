use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_identifier_definition_locations_interfaces() {
    let content = r#"var aa = 1;
interface /*interfaceName1*/
interface a/*interfaceName2*/"#;
    let mut s = Session::new_for_test("completionListAtIdentifierDefinitionLocations_interfaces", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), nil)
}
