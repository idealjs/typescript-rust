use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_infer_from_usage_rest_param3() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noImplicitAny: true
function f(a: number, [|...rest |]){
    a;
    rest.push(22);
}"#;
    let mut s = Session::new_for_test("codeFixInferFromUsageRestParam3", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `...rest: number[]`, false, 0, 0)
}
