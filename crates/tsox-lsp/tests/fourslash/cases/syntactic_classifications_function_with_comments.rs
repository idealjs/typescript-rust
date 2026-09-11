use tsox_lsp::fourslash::{self, Session};


#[test]
fn syntactic_classifications_function_with_comments() {
    let content = r#"/**
 * This is my function.
 * There are many like it, but this one is mine.
 */
function myFunction(/* x */ x: any) {
    var y = x ? x++ : ++x;
}
// end of file"#;
    let mut s = Session::new_for_test("syntacticClassificationsFunctionWithComments", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
