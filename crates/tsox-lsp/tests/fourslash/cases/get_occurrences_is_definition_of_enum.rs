use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn get_occurrences_is_definition_of_enum() {
    let content = r#"/*1*/enum /*2*/E {
    First,
    Second
}
let first = /*3*/E.First;"#;
    let mut s = Session::new_for_test("getOccurrencesIsDefinitionOfEnum", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
