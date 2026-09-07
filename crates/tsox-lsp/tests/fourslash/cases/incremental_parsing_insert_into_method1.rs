use tsox_lsp::fourslash::{self, Session};

#[test]
fn incremental_parsing_insert_into_method1() {
    let content = r#"class C {
    public foo1() { }
    public foo2() {
        return 1/*1*/;
    }
    public foo3() { }
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, " + 1");
}
