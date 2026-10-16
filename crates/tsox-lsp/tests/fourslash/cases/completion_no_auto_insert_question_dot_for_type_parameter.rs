use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_no_auto_insert_question_dot_for_type_parameter() {
    let content = r#"// @strict: true
interface Address {
    city: string = "";
    "postal code": string = "";
}
function f<T extends Address>(x: T) {
    x[|./**/|]
}"#;
    let mut s = Session::new_for_test("completionNoAutoInsertQuestionDotForTypeParameter", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
