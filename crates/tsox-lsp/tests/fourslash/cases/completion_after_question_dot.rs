use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_after_question_dot() {
    let content = r#"// @strict: true
class User {
    #foo: User;
    bar: User;
    address?: {
        city: string;
        "postal code": string;
    };
    constructor() {
        this.address[|?./*1*/|];
        this[|?./*2*/|];
        this?.bar[|?./*3*/|];
    }
};"#;
    let mut s = Session::new_for_test("completionAfterQuestionDot", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
