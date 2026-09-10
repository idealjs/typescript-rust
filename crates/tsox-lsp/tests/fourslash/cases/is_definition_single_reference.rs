use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn is_definition_single_reference() {
    let content = r#"function /*1*/f() {}
/*2*/f();"#;
    let mut s = Session::new_for_test("isDefinitionSingleReference", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
