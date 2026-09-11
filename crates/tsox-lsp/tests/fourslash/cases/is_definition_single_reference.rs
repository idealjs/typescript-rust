use tsox_lsp::fourslash::{self, Session};


#[test]
fn is_definition_single_reference() {
    let content = r#"function /*1*/f() {}
/*2*/f();"#;
    let mut s = Session::new_for_test("isDefinitionSingleReference", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
