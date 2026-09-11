use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_is_definition_of_namespace() {
    let content = r#"/*1*/namespace /*2*/Numbers {
    export var n = 12;
}
let x = /*3*/Numbers.n + 1;"#;
    let mut s = Session::new_for_test("getOccurrencesIsDefinitionOfNamespace", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
