use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_add_void_to_promise5() {
    let content = r#"// @target: esnext
// @lib: es2015
// @strict: true
const p4: Promise<number> = new Promise(resolve => resolve());"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
