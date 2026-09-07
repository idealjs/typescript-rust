use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_catch_info() {
    let content = r#"try {} catch(/*1*/error) {}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "var error: unknown", "");
}
