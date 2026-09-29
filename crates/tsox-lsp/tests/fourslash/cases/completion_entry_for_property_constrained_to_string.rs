use tsox_lsp::fourslash::Session;


#[test]
fn completion_entry_for_property_constrained_to_string() {
    let content = r#"declare function test<P extends "a" | "b">(p: { type: P }): void;

test({ type: /*ts*/ })"#;
    let _s = Session::new_for_test("completionEntryForPropertyConstrainedToString", content);
    // TODO: f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
}
