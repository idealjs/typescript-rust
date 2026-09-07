use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn semantic_classificaton_type_alias() {
    let content = r#"type /*0*/Alias = number
var x: /*1*/Alias;
var y = </*2*/Alias>{};
function f(x: /*3*/Alias): /*4*/Alias { return undefined; }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
