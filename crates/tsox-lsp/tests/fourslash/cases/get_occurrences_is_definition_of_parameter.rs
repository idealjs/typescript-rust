use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_is_definition_of_parameter() {
    let content = r#"function f(/*1*/x: number) {
  return /*2*/x + 1
}"#;
    let mut s = Session::new_for_test("getOccurrencesIsDefinitionOfParameter", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
