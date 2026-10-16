use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_is_definition_of_interface() {
    let content = r#"/*1*/interface /*2*/I {
    p: number;
}
let i: /*3*/I = { p: 12 };"#;
    let _s = Session::new_for_test("getOccurrencesIsDefinitionOfInterface", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
