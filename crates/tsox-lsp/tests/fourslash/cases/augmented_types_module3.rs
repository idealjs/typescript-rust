use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Backspace"]
#[test]
fn augmented_types_module3() {
    let content = r#"function m2g() { };
namespace m2g { export class C { foo(x: number) { } } }
var x: m2g./*1*/;
var /*2*/r = m2g/*3*/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "C.");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, nil)
    fourslash::unsupported("Backspace"); // f.Backspace(t, 1)
    fourslash::verify_quick_info_at(&mut s, "2", "var r: typeof m2g", "");
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("Insert"); // f.Insert(t, "(")
}
