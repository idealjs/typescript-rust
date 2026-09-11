use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_infer_from_usage_call() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noImplicitAny: true
function wat([|b |]) {
    b();
}"#;
    let mut s = Session::new_for_test("codeFixInferFromUsageCall", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `b: () => void`, false, 0, 0)
}
