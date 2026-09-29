use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_not_inside_comment() {
    let content = r#"a/* /**/ */.b"#;
    let mut s = Session::new_for_test("quickInfo_notInsideComment", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyNotQuickInfoExists(t)
}
