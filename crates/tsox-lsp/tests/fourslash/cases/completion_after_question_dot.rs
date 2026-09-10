use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_after_question_dot() {
    // TODO: t.Skip("Known failing fourslash test")
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
