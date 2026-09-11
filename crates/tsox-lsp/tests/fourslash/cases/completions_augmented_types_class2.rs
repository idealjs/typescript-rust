use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_augmented_types_class2() {
    let content = r#"class c5b { public foo(){ } }
namespace c5b { var y = 2; } // should be ok
c5b./*1*/
var r = new c5b();
r./*2*/"#;
    let mut s = Session::new_for_test("completionsAugmentedTypesClass2", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.Backspace(t, 4)
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
