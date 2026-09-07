use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_on_generic_class() {
    let content = r#"class Contai/**/ner<T> {
    x: T;
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "class Container<T>", "");
}
