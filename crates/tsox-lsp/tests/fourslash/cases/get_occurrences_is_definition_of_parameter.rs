use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_is_definition_of_parameter() {
    let content = r#"function f(/*1*/x: number) {
  return /*2*/x + 1
}"#;
    let _s = Session::new_for_test("getOccurrencesIsDefinitionOfParameter", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
