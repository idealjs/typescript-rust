use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_is_definition_of_type_alias() {
    let content = r#"/*1*/type /*2*/Alias= number;
let n: /*3*/Alias = 12;"#;
    let _s = Session::new_for_test("getOccurrencesIsDefinitionOfTypeAlias", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
