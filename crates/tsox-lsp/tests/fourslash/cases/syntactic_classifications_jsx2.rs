use tsox_lsp::fourslash::Session;


#[test]
fn syntactic_classifications_jsx2() {
    let content = r#"// @Filename: file1.tsx
let x  = <div.name b = "some-value" c = {1}>
    some jsx text
</div.name>;

let y = <element.name attr="123"/>"#;
    let _s = Session::new_for_test("syntacticClassificationsJsx2", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
