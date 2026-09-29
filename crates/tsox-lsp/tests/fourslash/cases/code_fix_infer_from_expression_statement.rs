use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_expression_statement() {
    let content = r#"// @noImplicitAny: true
function inferVoid( [| app |] ) {
    app.use('hi')
}"#;
    let _s = Session::new_for_test("codeFixInferFromExpressionStatement", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `app: { use: (arg0: string) => void; }`, false, 0, 0)
}
