use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_infer_from_usage_no_crash_on_missing_parens() {
    let content = r#"// @noImplicitAny: true
// @target: esnext
class C {
    m() { this.x * 2; }
    get x { return null; }
}"#;
    let _s = Session::new_for_test("codeFixInferFromUsage_noCrashOnMissingParens", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
