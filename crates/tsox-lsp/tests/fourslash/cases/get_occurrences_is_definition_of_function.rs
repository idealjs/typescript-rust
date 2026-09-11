use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_is_definition_of_function() {
    let content = r#"/*1*/function /*2*/func(x: number) {
}
/*3*/func(x)"#;
    let mut s = Session::new_for_test("getOccurrencesIsDefinitionOfFunction", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
