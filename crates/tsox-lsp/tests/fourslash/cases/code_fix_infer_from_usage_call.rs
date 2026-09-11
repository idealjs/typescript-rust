use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_usage_call() {
    let content = r#"// @noImplicitAny: true
function wat([|b |]) {
    b();
}"#;
    let mut s = Session::new_for_test("codeFixInferFromUsageCall", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `b: () => void`, false, 0, 0)
}
