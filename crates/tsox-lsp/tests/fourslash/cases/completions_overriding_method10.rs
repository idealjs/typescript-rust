use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_method10() {
    let content = r#"// @Filename: a.ts
// @newline: LF
interface Base {
    a: string;
    b(a: string): void;
    c(a: string): string;
    c(a: number): number;
}
class Sub implements Base {
   /*a*/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod10", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
