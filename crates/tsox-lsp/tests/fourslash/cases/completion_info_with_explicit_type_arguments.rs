use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_info_with_explicit_type_arguments() {
    let content = r#"interface I { x: number; y: number; }

declare function f<T>(x: T, y: number): void;
f<I>({ /*f*/ });

declare function g<T>(x: keyof T, y: number): void;
g<I>("[|/*g*/|]");"#;
    let mut s = Session::new_for_test("completionInfoWithExplicitTypeArguments", content);
    fourslash::go_to_marker(&mut s, "f");
    // TODO: f.VerifyCompletions(t, "f", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "g");
    // TODO: f.VerifyCompletions(t, "g", &fourslash.CompletionsExpectedList{
}
