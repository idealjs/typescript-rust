use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_entry_for_argument_constrained_to_string() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"declare function test<P extends "a" | "b">(p: P): void;

test(/*ts*/)
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
}
