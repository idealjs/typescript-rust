use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_add_missing_member21() {
    let content = r#"declare let p: Promise<string>;
async function f() {
    p.toLowerCase();
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "fixMissingMember")
}
