use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_generic_string_literal() {
    let content = r#"// @strict: true
declare function get<T, K extends keyof T>(obj: T, key: K): T[K];
get({ hello: 123, world: 456 }, "/**/");"#;
    let mut s = Session::new_for_test("completionsWithGenericStringLiteral", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["hello", "world"], &[]);
}
