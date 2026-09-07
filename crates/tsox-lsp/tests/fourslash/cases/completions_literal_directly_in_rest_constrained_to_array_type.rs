use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_literal_directly_in_rest_constrained_to_array_type() {
    let content = r#"// @strict: true

function fn<T extends ('value1' | 'value2' | 'value3')[]>(...values: T): T { return values; }

const value1 = fn('/*1*/');
const value2 = fn('value1', '/*2*/');"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
