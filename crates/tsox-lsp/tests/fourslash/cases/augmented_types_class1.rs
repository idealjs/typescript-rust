use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn augmented_types_class1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class c5b { public foo() { } }
namespace c5b { export var y = 2; } // should be ok
c5b./*1*/
var r = new c5b();
r./*2*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "y;");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
