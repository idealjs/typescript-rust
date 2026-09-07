use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Backspace"]
#[test]
fn augmented_types_module2() {
    let content = r#"function /*11*/m2f(x: number) { };
namespace m2f { export interface I { foo(): void } }
var x: m2f./*1*/
var /*2*/r = m2f/*3*/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "11", "function m2f(x: number): void", "")
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "I.");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, nil)
    fourslash::unsupported("Backspace"); // f.Backspace(t, 1)
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var r: (x: number) => void", "")
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("Insert"); // f.Insert(t, "(")
}
