use tsox_lsp::fourslash::{self, Session};


#[test]
fn goto_definition_throws_tag() {
    let content = r#"class [|/*def*/E|] extends Error {}

/**
 * @throws {/*use*/[|E|]}
 */
function f() {}"#;
    let mut s = Session::new_for_test("gotoDefinitionThrowsTag", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "use")
}
