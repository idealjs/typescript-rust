use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.Backspace"]
#[test]
fn completions_augmented_types_class2() {
    let content = r#"class c5b { public foo(){ } }
namespace c5b { var y = 2; } // should be ok
c5b./*1*/
var r = new c5b();
r./*2*/"#;
    let mut s = Session::new_for_test("completionsAugmentedTypesClass2", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("Backspace"); // f.Backspace(t, 4)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
