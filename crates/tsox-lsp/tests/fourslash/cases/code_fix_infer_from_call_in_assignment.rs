use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_call_in_assignment() {
    let content = r#"// @noImplicitAny: true
function inferAny( [| app |] ) {
    const result = app.use('hi')
    return result
}"#;
    let mut s = Session::new_for_test("codeFixInferFromCallInAssignment", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `app: { use: (arg0: string) => any }`, false, 0, 0)
}
