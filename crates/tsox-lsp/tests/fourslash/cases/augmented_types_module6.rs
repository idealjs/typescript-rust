use tsox_lsp::fourslash::{self, Session};


#[test]
fn augmented_types_module6() {
    let content = r#"declare class m3f { foo(x: number): void }
namespace m3f { export interface I { foo(): void } }
var x: m3f./*1*/
var /*4*/r = new /*2*/m3f(/*3*/);
r./*5*/
var r2: m3f.I = r;
r2./*6*/"#;
    let mut s = Session::new_for_test("augmentedTypesModule6", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "I;");
    fourslash::verify_completions_include_exclude_at(&mut s, Some("2"), &["m3f"], &[]);
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "m3f(): m3f"})
    fourslash::verify_quick_info_at(&mut s, "4", "var r: m3f", "");
    fourslash::verify_completions_include_exclude_at(&mut s, Some("5"), &["foo"], &[]);
    fourslash::insert(&mut s, "foo(1)");
    fourslash::verify_completions_include_exclude_at(&mut s, Some("6"), &["foo"], &[]);
    fourslash::insert(&mut s, "foo(");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(): void"})
}
