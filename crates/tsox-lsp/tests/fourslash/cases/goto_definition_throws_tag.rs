use tsox_lsp::fourslash::Session;


#[test]
fn goto_definition_throws_tag() {
    let content = r#"class [|/*def*/E|] extends Error {}

/**
 * @throws {/*use*/[|E|]}
 */
function f() {}"#;
    let _s = Session::new_for_test("gotoDefinitionThrowsTag", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "use")
}
