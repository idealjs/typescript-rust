use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_method11() {
    let content = r#"// @Filename: a.ts
// @newline: LF
function foo() {
    const a = 1
    const b = 2
    foo()
    return a + b
}

interface Base {
    a: string
    b(a: string): void
    c(a: string): string
    c(a: number): number
}
class Sub implements Base {
   /*a*/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod11", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
