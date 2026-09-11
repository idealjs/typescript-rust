use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_identifier_definition_locations_infers() {
    let content = r#"type UType = 1;
type Bar<T> = T extends { a: (x: infer /*1*/) => void; b: (x: infer U/*2*/) => void }
   ? U
   : never;"#;
    let mut s = Session::new_for_test("completionListAtIdentifierDefinitionLocations_infers", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), nil)
}
