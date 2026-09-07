use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyRenameFailed"]
#[test]
fn get_rename_info_tests2() {
    let content = r#"class C /**/extends null {

}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyRenameFailed"); // f.VerifyRenameFailed(t, nil /*preferences*/)
}
