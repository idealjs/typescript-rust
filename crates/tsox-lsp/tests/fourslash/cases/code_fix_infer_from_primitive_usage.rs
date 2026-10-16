use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_primitive_usage() {
    let content = r#"// @noImplicitAny: true
function wrap( [| s |] ) {
    return s.length + s.indexOf('hi')
}"#;
    let _s = Session::new_for_test("codeFixInferFromPrimitiveUsage", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `s: string | string[]`, false, 0, 0)
}
