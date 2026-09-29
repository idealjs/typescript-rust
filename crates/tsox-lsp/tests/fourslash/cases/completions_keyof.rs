use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_keyof() {
    let content = r#"interface A { a: number; };
interface B { a: number; b: number; };
function f<T extends keyof A>(key: T) {}
f("[|/*f*/|]");
function g<T extends keyof B>(key: T) {}
g("[|/*g*/|]");"#;
    let mut s = Session::new_for_test("completionsKeyof", content);
    fourslash::go_to_marker(&mut s, "f");
    // TODO: f.VerifyCompletions(t, "f", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "g");
    // TODO: f.VerifyCompletions(t, "g", &fourslash.CompletionsExpectedList{
}
