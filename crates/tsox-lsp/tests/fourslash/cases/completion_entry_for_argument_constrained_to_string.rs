use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_entry_for_argument_constrained_to_string() {
    let content = r#"declare function test<P extends "a" | "b">(p: P): void;

test(/*ts*/)
"#;
    let _s = Session::new_for_test("completionEntryForArgumentConstrainedToString", content);
    // TODO: f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
}
