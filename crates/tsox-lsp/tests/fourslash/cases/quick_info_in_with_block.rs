use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_in_with_block() {
    let content = r#"with (x) {
    function /*1*/f() { }
    var /*2*/b = /*3*/f;
}"#;
    let mut s = Session::new_for_test("quickInfoInWithBlock", content);
    fourslash::verify_quick_info_at(&mut s, "1", "any", "");
    fourslash::verify_quick_info_at(&mut s, "2", "any", "");
    fourslash::verify_quick_info_at(&mut s, "3", "any", "");
}
