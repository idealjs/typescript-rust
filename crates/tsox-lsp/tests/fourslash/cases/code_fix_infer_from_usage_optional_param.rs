use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_usage_optional_param() {
    let content = r#"// @strict: false
// @noImplicitAny: true
function f([|a? |]){
    a;
}
f();
f(1);"#;
    let _s = Session::new_for_test("codeFixInferFromUsageOptionalParam", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `a?: number`, false, 0, 0)
}
