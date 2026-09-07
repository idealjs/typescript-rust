use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn goto_definition_throws_tag() {
    let content = r#"class [|/*def*/E|] extends Error {}

/**
 * @throws {/*use*/[|E|]}
 */
function f() {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "use")
}
