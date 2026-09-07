use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_entry_for_property_constrained_to_string() {
    let content = r#"declare function test<P extends "a" | "b">(p: { type: P }): void;

test({ type: /*ts*/ })"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
}
