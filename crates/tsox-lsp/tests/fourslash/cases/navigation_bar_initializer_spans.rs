use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_initializer_spans() {
    let content = r#"// get the name for the navbar from the variable name rather than the function name
const [|[|x|] = () => { var [|a|]; }|];
const [|[|f|] = function f() { var [|b|]; }|];
const [|[|y|] = { [|[|z|]: function z() { var [|c|]; }|] }|];"#;
    let _s = Session::new_for_test("navigationBarInitializerSpans", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
