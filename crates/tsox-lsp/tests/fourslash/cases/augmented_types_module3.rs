use tsox_lsp::fourslash::{self, Session};


#[test]
fn augmented_types_module3() {
    let content = r#"function m2g() { };
namespace m2g { export class C { foo(x: number) { } } }
var x: m2g./*1*/;
var /*2*/r = m2g/*3*/;"#;
    let mut s = Session::new_for_test("augmentedTypesModule3", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["C"]);
    fourslash::insert(&mut s, "C.");
    fourslash::verify_completions_empty_at(&mut s, None);
    // TODO: f.Backspace(t, 1)
    fourslash::verify_quick_info_at(&mut s, "2", "var r: typeof m2g", "");
    fourslash::go_to_marker(&mut s, "3");
    fourslash::insert(&mut s, "(");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "m2g(): void"})
}
