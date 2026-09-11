use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_infer_from_expression_statement() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noImplicitAny: true
function inferVoid( [| app |] ) {
    app.use('hi')
}"#;
    let mut s = Session::new_for_test("codeFixInferFromExpressionStatement", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `app: { use: (arg0: string) => void; }`, false, 0, 0)
}
