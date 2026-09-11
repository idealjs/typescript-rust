use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_auto_insert_question_dot() {
    let content = r#"// @strict: true
interface User {
    address?: {
        city: string;
        "postal code": string;
    }
};
declare const user: User;
user.address[|./**/|]"#;
    let mut s = Session::new_for_test("completionAutoInsertQuestionDot", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
