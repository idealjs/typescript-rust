use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_entry_for_array_element_constrained_to_string() {
    let content = r#"declare function test<T extends 'a' | 'b'>(a: { foo: T[] }): void

test({ foo: [/*ts*/] })"#;
    let mut s = Session::new_for_test("completionEntryForArrayElementConstrainedToString", content);
    // TODO: f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
}
