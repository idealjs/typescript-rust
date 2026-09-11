use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_is_definition_of_binding_pattern() {
    let content = r#"const { /*1*/x, y } = { /*2*/x: 1, y: 2 };
const z = /*3*/x;"#;
    let mut s = Session::new_for_test("getOccurrencesIsDefinitionOfBindingPattern", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
