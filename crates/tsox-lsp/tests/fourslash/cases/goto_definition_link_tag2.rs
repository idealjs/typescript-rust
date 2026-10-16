use tsox_lsp::fourslash::Session;


#[test]
fn goto_definition_link_tag2() {
    let content = r#"enum E {
    /** {@link /*1*/[|A|]} */
    [|/*2*/A|]
}"#;
    let _s = Session::new_for_test("gotoDefinitionLinkTag2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "1")
}
