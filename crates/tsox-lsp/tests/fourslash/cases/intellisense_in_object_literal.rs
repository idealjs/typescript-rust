use tsox_lsp::fourslash::{self, Session};

#[test]
fn intellisense_in_object_literal() {
    let content = r#"var x = 3;

class Foo {
    static something() {
        return { "prop": /**/x };
    }
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "var x: number", "");
}
