use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal_from_signature() {
    let content = r#"declare function f(a: "x"): void;
declare function f(a: string): void;
f("[|/**/|]");"#;
    let mut s = Session::new_for_test("completionForStringLiteralFromSignature", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
