use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_is_definition_of_number_named_property() {
    let content = r#"let o = { /*1*/1: 12 };
let y = o[/*2*/1];"#;
    let mut s = Session::new_for_test("getOccurrencesIsDefinitionOfNumberNamedProperty", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
