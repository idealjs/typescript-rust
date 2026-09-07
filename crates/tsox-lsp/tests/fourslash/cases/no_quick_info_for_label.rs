use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNotQuickInfoExists"]
#[test]
fn no_quick_info_for_label() {
    let content = r#"/*1*/label : while(true){
    break /*2*/label;
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyNotQuickInfoExists"); // f.VerifyNotQuickInfoExists(t)
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifyNotQuickInfoExists"); // f.VerifyNotQuickInfoExists(t)
}
