use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_infer_from_usage_no_crash_on_missing_parens() {
    let content = r#"// @noImplicitAny: true
// @target: esnext
class C {
    m() { this.x * 2; }
    get x { return null; }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
