use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn augmented_types_class1() {
    let content = r#"class c5b { public foo() { } }
namespace c5b { export var y = 2; } // should be ok
c5b./*1*/
var r = new c5b();
r./*2*/"#;
    let mut s = Session::new_for_test("augmentedTypesClass1", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "y;");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
