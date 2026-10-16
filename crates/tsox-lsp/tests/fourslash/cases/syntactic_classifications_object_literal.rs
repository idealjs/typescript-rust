use tsox_lsp::fourslash::Session;


#[test]
fn syntactic_classifications_object_literal() {
    let content = r#"var v = 10e0;
var x = {
    p1: 1,
    p2: 2,
    any: 3,
    function: 4,
    var: 5,
    void: void 0,
    v: v += v,
};"#;
    let _s = Session::new_for_test("syntacticClassificationsObjectLiteral", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
