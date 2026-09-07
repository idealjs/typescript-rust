use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_no_auto_insert_question_dot_with_user_preferences_off() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: true
interface User {
    address?: {
        city: string;
        "postal code": string;
    }
};
declare const user: User;
user.address[|./**/|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
