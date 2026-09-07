use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Insert"]
#[test]
fn augmented_types_module6() {
    let content = r#"declare class m3f { foo(x: number): void }
namespace m3f { export interface I { foo(): void } }
var x: m3f./*1*/
var /*4*/r = new /*2*/m3f(/*3*/);
r./*5*/
var r2: m3f.I = r;
r2./*6*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "I;");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "m3f(): m3f"})
    fourslash::verify_quick_info_at(&mut s, "4", "var r: m3f", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "foo(1)");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("Insert"); // f.Insert(t, "foo(")
}
