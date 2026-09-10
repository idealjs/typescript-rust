use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn completion_list_in_object_literal_that_is_parameter_of_function_call() {
    let content = r#"function f(a: { xa: number; xb: number; }) { }
var xc;
f({
    /**/"#;
    let mut s = Session::new_for_test("completionListInObjectLiteralThatIsParameterOfFunctionCall", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["xa", "xb"]);
    // TODO: }
}
