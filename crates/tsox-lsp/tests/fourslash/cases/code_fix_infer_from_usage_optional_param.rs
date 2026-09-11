use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_infer_from_usage_optional_param() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
// @noImplicitAny: true
function f([|a? |]){
    a;
}
f();
f(1);"#;
    let mut s = Session::new_for_test("codeFixInferFromUsageOptionalParam", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `a?: number`, false, 0, 0)
}
