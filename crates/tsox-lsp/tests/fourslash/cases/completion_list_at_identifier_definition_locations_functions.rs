use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_identifier_definition_locations_functions() {
    let content = r#"var aa = 1;
function /*functionName1*/
function a/*functionName2*/"#;
    let mut s = Session::new_for_test("completionListAtIdentifierDefinitionLocations_functions", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), nil)
}
