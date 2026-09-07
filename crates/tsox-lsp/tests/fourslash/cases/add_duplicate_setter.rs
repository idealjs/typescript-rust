use tsox_lsp::fourslash::{self, Session};

#[test]
fn add_duplicate_setter() {
    let content = r#"class C {
    set foo(value) { }
    /**/
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "set foo(value) { }");
}
