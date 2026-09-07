use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_add_void_to_promise_js5() {
    let content = r#"// @target: esnext
// @lib: es2015
// @strict: true
// @allowJS: true
// @checkJS: true
// @filename: main.js
/** @type {Promise<number>} */
const p2 = new Promise(resolve => resolve());"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "Add 'void' to Promise resolved without a value")
}
