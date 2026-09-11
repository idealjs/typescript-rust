use tsox_lsp::fourslash::{self, Session};


#[test]
fn goto_definition_link_tag5() {
    let content = r#"enum E {
    /** {@link /*1*/[|B|]} */
    A,
    [|/*2*/B|]
}"#;
    let mut s = Session::new_for_test("gotoDefinitionLinkTag5", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "1")
}
