use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal5() {
    let content = r#"// @stableTypeOrdering: true
interface Foo {
    foo: string;
    bar: string;
}

function f<K extends keyof Foo>(a: K) { };
f("/*1*/"#;
    let mut s = Session::new_for_test("completionForStringLiteral5", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["bar", "foo"]);
}
