use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_primitive_usage() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noImplicitAny: true
function wrap( [| s |] ) {
    return s.length + s.indexOf('hi')
}"#;
    let mut s = Session::new_for_test("codeFixInferFromPrimitiveUsage", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `s: string | string[]`, false, 0, 0)
}
