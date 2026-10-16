use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_is_definition_of_arrow_function() {
    let content = r#"/*1*/var /*2*/f = x => x + 1;
/*3*/f(12);"#;
    let _s = Session::new_for_test("getOccurrencesIsDefinitionOfArrowFunction", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
