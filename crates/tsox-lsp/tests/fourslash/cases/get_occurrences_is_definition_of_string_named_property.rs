use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_is_definition_of_string_named_property() {
    let content = r#"let o = { /*1*/"/*2*/x": 12 };
let y = o./*3*/x;"#;
    let _s = Session::new_for_test("getOccurrencesIsDefinitionOfStringNamedProperty", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
