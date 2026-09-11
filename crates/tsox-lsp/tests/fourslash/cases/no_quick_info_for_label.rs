use tsox_lsp::fourslash::{self, Session};


#[test]
fn no_quick_info_for_label() {
    let content = r#"/*1*/label : while(true){
    break /*2*/label;
}"#;
    let mut s = Session::new_for_test("noQuickInfoForLabel", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyNotQuickInfoExists(t)
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyNotQuickInfoExists(t)
}
