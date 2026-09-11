use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal_from_signature2() {
    let content = r#"declare function f(a: "x"): void;
declare function f(a: string, b: number): void;
f("/**/", 0);"#;
    let mut s = Session::new_for_test("completionForStringLiteralFromSignature2", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
