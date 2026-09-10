use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn goto_definition_link_tag2() {
    let content = r#"enum E {
    /** {@link /*1*/[|A|]} */
    [|/*2*/A|]
}"#;
    let mut s = Session::new_for_test("gotoDefinitionLinkTag2", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "1")
}
