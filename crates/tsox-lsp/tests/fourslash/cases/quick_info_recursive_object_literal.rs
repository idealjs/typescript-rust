use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn quick_info_recursive_object_literal() {
    let content = r#"var a = { f: /**/a"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "var a: any", "");
    // TODO: }
}
