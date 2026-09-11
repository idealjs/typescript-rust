use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_of_enum_in_module() {
    let content = r#"namespace Fixes {
    enum Foo {
        bar,
        baz
    }
    var f: Foo = Foo./**/;
}"#;
    let mut s = Session::new_for_test("memberListOfEnumInModule", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["bar", "baz"]);
}
