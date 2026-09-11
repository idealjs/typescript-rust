use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_generic_with_constraints1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface Fo/*1*/o<T/*2*/T extends Date> {}"#;
    let mut s = Session::new_for_test("quickInfoOnGenericWithConstraints1", content);
    fourslash::verify_quick_info_at(&mut s, "1", "interface Foo<TT extends Date>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(type parameter) TT in Foo<TT extends Date>", "");
}
