use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_function() {
    let content = r#"// Outside the function
declare function f1(a: number);/*1*/

// inside the function
declare function f2(b: number, b2 = /*2*/

// Outside the function
function f3(c: number) { }/*3*/

// inside the function
function f4(d: number) { /*4*/}"#;
    let mut s = Session::new_for_test("completionListAfterFunction", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_include_exclude_at(&mut s, Some("2"), &["b"], &[]);
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_include_exclude_at(&mut s, Some("4"), &["d"], &[]);
}
