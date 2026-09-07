use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_initializer_spans() {
    let content = r#"// get the name for the navbar from the variable name rather than the function name
const [|[|x|] = () => { var [|a|]; }|];
const [|[|f|] = function f() { var [|b|]; }|];
const [|[|y|] = { [|[|z|]: function z() { var [|c|]; }|] }|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
