use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_add_missing_member21() {
    let content = r#"declare let p: Promise<string>;
async function f() {
    p.toLowerCase();
}"#;
    let mut s = Session::new_for_test("codeFixAddMissingMember21", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixMissingMember")
}
