use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_no_auto_insert_question_dot_for_this() {
    let content = r#"// @strict: true
class Address {
    city: string = "";
    "postal code": string = "";
    method() {
        this[|./**/|]
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
