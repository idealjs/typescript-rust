use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
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
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `a?: number`, false, 0, 0)
}
