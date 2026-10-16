use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_is_definition_of_enum() {
    let content = r#"/*1*/enum /*2*/E {
    First,
    Second
}
let first = /*3*/E.First;"#;
    let _s = Session::new_for_test("getOccurrencesIsDefinitionOfEnum", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
