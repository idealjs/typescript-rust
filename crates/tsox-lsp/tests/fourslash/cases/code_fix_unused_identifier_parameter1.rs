use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_unused_identifier_parameter1() {
    let content = r#"// @noUnusedLocals: true
// @noUnusedParameters: true
function g(a, b) { b; }
g(1, 2);"#;
    let mut s = Session::new_for_test("codeFixUnusedIdentifier_parameter1", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "Remove unused declaration for: 'a'")
}
