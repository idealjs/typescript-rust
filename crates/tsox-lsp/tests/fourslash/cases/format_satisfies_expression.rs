use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_satisfies_expression() {
    let content = r#"type Foo = "a" | "b" | "c";
const foo1 = ["a"] satisfies Foo[];
const foo2 = ["a"]satisfies Foo[];
const foo3 = ["a"]  satisfies Foo[];"#;
    let mut s = Session::new_for_test("formatSatisfiesExpression", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"type Foo = "a" | "b" | "c";
const foo1 = ["a"] satisfies Foo[];
const foo2 = ["a"] satisfies Foo[];
const foo3 = ["a"] satisfies Foo[];"#);
}
