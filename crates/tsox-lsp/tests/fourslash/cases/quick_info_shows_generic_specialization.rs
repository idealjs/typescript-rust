use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_shows_generic_specialization() {
    let content = r#"class A<T> { }
var /**/foo = new A<number>();"#;
    let mut s = Session::new_for_test("quickInfoShowsGenericSpecialization", content);
    fourslash::verify_quick_info_at(&mut s, "", "var foo: A<number>", "");
}
