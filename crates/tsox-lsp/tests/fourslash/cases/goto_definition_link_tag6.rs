use tsox_lsp::fourslash::Session;


#[test]
fn goto_definition_link_tag6() {
    let content = r#"enum E {
    /** {@link E./*1*/[|A|]} */
    [|/*2*/A|]
}"#;
    let _s = Session::new_for_test("gotoDefinitionLinkTag6", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "1")
}
