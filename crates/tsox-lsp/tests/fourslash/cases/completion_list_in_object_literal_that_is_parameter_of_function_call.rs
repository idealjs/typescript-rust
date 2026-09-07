use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn completion_list_in_object_literal_that_is_parameter_of_function_call() {
    let content = r#"function f(a: { xa: number; xb: number; }) { }
var xc;
f({
    /**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: }
}
