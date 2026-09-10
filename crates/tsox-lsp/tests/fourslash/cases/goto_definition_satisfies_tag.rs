use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn goto_definition_satisfies_tag() {
    let content = r#"// @noEmit: true
// @allowJS: true
// @checkJs: true
// @filename: /a.js
/**
 * @typedef {Object} [|/*def*/T|]
 * @property {number} a
 */

/** @satisfies {/*use*/[|T|]} comment */
const foo = { a: 1 };"#;
    let mut s = Session::new_for_test("gotoDefinitionSatisfiesTag", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "use")
}
