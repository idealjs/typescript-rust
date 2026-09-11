use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_rename_info_tests1() {
    let content = r#"class [|/**/C|] {

}"#;
    let mut s = Session::new_for_test("getRenameInfoTests1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
