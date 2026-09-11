use tsox_lsp::fourslash::{self, Session};


#[test]
fn syntactic_classifications_templates2() {
    let content = r#"var tiredOfCanonicalExamples =
` + "`" + `goodbye "${ ` + "`" + `hello world` + "`" + "#;
    // TODO: and ${ ` + "`" + `good${ " " }riddance` + "`" + ` }` + "`" + `;`
    let mut s = Session::new_for_test("syntacticClassificationsTemplates2", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
