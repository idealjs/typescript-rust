use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_index_signature2() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
const o = {};
o.[|/*use*/foo|];"#;
    let mut s = Session::new_for_test("goToDefinitionIndexSignature2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "use")
}
