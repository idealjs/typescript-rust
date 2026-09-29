use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_override1() {
    let content = r#"class A {
    foo () {} 
    bar () {}
}
class B extends A {
    override /*1*/
}"#;
    let mut s = Session::new_for_test("completionsWithOverride1", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["foo", "bar"], &[]);
}
