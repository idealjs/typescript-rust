use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_const_assertion() {
    let content = r#"const foo = 42 as /*1*/const"#;
    let mut s = Session::new_for_test("quickInfoConstAssertion", content);
    fourslash::verify_quick_info_at(&mut s, "1", "type const = 42", "");
}
