use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn get_occurrences_is_definition_of_arrow_function() {
    let content = r#"/*1*/var /*2*/f = x => x + 1;
/*3*/f(12);"#;
    let mut s = Session::new_for_test("getOccurrencesIsDefinitionOfArrowFunction", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
