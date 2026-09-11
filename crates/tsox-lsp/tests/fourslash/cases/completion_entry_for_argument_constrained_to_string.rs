use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_entry_for_argument_constrained_to_string() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"declare function test<P extends "a" | "b">(p: P): void;

test(/*ts*/)
"#;
    let mut s = Session::new_for_test("completionEntryForArgumentConstrainedToString", content);
    // TODO: f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
}
