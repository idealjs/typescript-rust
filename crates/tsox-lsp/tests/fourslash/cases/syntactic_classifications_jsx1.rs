use tsox_lsp::fourslash::Session;


#[test]
fn syntactic_classifications_jsx1() {
    let content = r#"// @Filename: file1.tsx
let x  = <div a = "some-value" b = {1}>
    some jsx text
</div>;

let y = <element attr="123"/>"#;
    let _s = Session::new_for_test("syntacticClassificationsJsx1", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
