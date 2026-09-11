use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal3() {
    let content = r#"declare function f(a: "A", b: number): void;
declare function f(a: "B", b: number): void;
declare function f(a: "C", b: number): void;
declare function f(a: string, b: number): void;

f("[|/*1*/C|]", 2);

f("/*2*/"#;
    let mut s = Session::new_for_test("completionForStringLiteral3", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["A", "B", "C"]);
}
