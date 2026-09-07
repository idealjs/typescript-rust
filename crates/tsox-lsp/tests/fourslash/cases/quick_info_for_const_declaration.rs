use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_for_const_declaration() {
    let content = r#"const /**/c = 0 ;"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "const c: 0", "");
}
