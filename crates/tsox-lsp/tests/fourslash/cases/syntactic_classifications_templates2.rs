use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: and ${ ` + '`' + `good${ ' ' }riddance` + '`' + ` }` + '`' +"]
#[test]
fn syntactic_classifications_templates2() {
    let content = r#"var tiredOfCanonicalExamples =
` + "`" + `goodbye "${ ` + "`" + `hello world` + "`" + "#;
    // TODO: and ${ ` + "`" + `good${ " " }riddance` + "`" + ` }` + "`" + `;`
    let mut s = Session::new_for_test("syntacticClassificationsTemplates2", content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
