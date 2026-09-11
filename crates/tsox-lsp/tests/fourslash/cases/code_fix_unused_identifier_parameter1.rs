use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_unused_identifier_parameter1() {
    let content = r#"// @noUnusedLocals: true
// @noUnusedParameters: true
function g(a, b) { b; }
g(1, 2);"#;
    let mut s = Session::new_for_test("codeFixUnusedIdentifier_parameter1", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "Remove unused declaration for: 'a'")
}
