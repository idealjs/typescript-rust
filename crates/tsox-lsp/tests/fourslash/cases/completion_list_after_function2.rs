use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_after_function2() {
    let content = r#"// Outside the function expression
declare var f1: (a: number) => void; /*1*/

declare var f1: (b: number, b2: /*2*/) => void;"#;
    let mut s = Session::new_for_test("completionListAfterFunction2", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "typeof ");
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["b"], &[]);
}
