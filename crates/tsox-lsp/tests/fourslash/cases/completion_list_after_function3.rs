use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_function3() {
    let content = r#"// Outside the function expression
var x1 = (a: number) => { }/*1*/;

var x2 = (b: number) => {/*2*/ };"#;
    let mut s = Session::new_for_test("completionListAfterFunction3", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_include_exclude_at(&mut s, Some("2"), &["b"], &[]);
}
