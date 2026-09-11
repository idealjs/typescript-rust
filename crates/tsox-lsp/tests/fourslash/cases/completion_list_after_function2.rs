use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_function2() {
    let content = r#"// Outside the function expression
declare var f1: (a: number) => void; /*1*/

declare var f1: (b: number, b2: /*2*/) => void;"#;
    let mut s = Session::new_for_test("completionListAfterFunction2", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "typeof ");
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["b"], &[]);
}
