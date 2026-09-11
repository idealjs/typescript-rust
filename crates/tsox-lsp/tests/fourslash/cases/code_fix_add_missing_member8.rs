use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_add_missing_member8() {
    let content = r#"// @Filename: a.ts
declare var x: [1, 2];
x.b;"#;
    let mut s = Session::new_for_test("codeFixAddMissingMember8", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
