use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_union_string_literal_property() {
    let content = r#"type Foo = { a: 0, b: 'x' } | { a: 0, b: 'y' } | { a: 1, b: 'z' };
const foo: Foo = { a: 0, b: '/*1*/' }

type Bar = { a: 0, b: 'fx' } | { a: 0, b: 'fy' } | { a: 1, b: 'fz' };
const bar: Bar = { a: 0, b: 'f/*2*/' }

type Baz = { x: 0, y: 0, z: 'a' } | { x: 0, y: 1, z: 'b' } | { x: 1, y: 0, z: 'c' } | { x: 1, y: 1, z: 'd' };
const baz1: Baz = { z: '/*3*/' };
const baz2: Baz = { x: 0, z: '/*4*/' };
const baz3: Baz = { x: 0, y: 1, z: '/*5*/' };
const baz4: Baz = { x: 2, y: 1, z: '/*6*/' };"#;
    let mut s = Session::new_for_test("completionsUnionStringLiteralProperty", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_include_exclude_at(&mut s, Some("3"), &["a", "b", "c", "d"], &[]);
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
}
