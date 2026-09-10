use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn get_occurrences_is_definition_of_interface() {
    let content = r#"/*1*/interface /*2*/I {
    p: number;
}
let i: /*3*/I = { p: 12 };"#;
    let mut s = Session::new_for_test("getOccurrencesIsDefinitionOfInterface", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
