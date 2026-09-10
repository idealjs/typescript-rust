use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn get_occurrences_is_definition_of_type_alias() {
    let content = r#"/*1*/type /*2*/Alias= number;
let n: /*3*/Alias = 12;"#;
    let mut s = Session::new_for_test("getOccurrencesIsDefinitionOfTypeAlias", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
