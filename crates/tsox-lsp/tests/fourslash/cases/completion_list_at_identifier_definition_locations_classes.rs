use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_identifier_definition_locations_classes() {
    let content = r#"var aa = 1;
class /*className1*/
class a/*className2*/"#;
    let mut s = Session::new_for_test("completionListAtIdentifierDefinitionLocations_classes", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), nil)
}
