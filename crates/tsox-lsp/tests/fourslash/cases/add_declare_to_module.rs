use tsox_lsp::fourslash::{self, Session};

#[test]
fn add_declare_to_module() {
    let content = r#"/**/namespace mAmbient {
    namespace m3 { }
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "declare ");
}
