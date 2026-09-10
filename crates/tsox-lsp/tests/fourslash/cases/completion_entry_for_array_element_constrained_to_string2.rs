use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_entry_for_array_element_constrained_to_string2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"declare function test<T extends 'a' | 'b'>(a: { foo: T[] }): void

test({ foo: ['a', /*ts*/] })"#;
    let mut s = Session::new_for_test("completionEntryForArrayElementConstrainedToString2", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
}
