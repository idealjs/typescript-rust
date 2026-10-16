use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_rename_info_tests2() {
    let content = r#"class C /**/extends null {

}"#;
    let mut s = Session::new_for_test("getRenameInfoTests2", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyRenameFailed(t, nil /*preferences*/)
}
