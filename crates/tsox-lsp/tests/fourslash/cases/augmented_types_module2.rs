use tsox_lsp::fourslash::{self, Session};


#[test]
fn augmented_types_module2() {
    let content = r#"function /*11*/m2f(x: number) { };
namespace m2f { export interface I { foo(): void } }
var x: m2f./*1*/
var /*2*/r = m2f/*3*/;"#;
    let mut s = Session::new_for_test("augmentedTypesModule2", content);
    fourslash::verify_quick_info_at(&mut s, "11", "function m2f(x: number): void", "");
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["I"]);
    fourslash::insert(&mut s, "I.");
    fourslash::verify_completions_empty_at(&mut s, None);
    // TODO: f.Backspace(t, 1)
    fourslash::verify_quick_info_at(&mut s, "2", "var r: (x: number) => void", "");
    fourslash::go_to_marker(&mut s, "3");
    fourslash::insert(&mut s, "(");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "m2f(x: number): void"})
}
