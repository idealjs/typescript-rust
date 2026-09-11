use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_usage_optional_param2() {
    let content = r#"// @noImplicitAny: true
function f([|a? |]){
    if (a < 9) return;
}"#;
    let mut s = Session::new_for_test("codeFixInferFromUsageOptionalParam2", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `a?: number`, false, 0, 0)
}
