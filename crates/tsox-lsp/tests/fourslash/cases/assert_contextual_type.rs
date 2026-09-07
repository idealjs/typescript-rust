use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn assert_contextual_type() {
    let content = r#"<(aa: number) =>void >(function myFn(b/**/b) { });"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(parameter) bb: number", "")
}
