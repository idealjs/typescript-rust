use tsox_lsp::fourslash::{self, Session};

#[test]
fn regexp() {
    let content = r#"var /**/x = /aa/;"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "var x: RegExp", "");
}
