use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: IsIncomplete: false,"]
#[test]
fn multi_module_fundule() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
function C(x: number) { }

namespace C {
    export var x = 1;
}
namespace C {
    export function foo() { }
}

var /*2*/r = C(/*1*/
var /*4*/r2 = new C(/*3*/ // using void returning function as constructor
var r3 = C./*5*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.Insert(t, "C.x);")
    // TODO: IsIncomplete: false,
    // TODO: ItemDefaults: &fourslash.CompletionsExpectedItemDefaults{
    // TODO: Items: &fourslash.CompletionsExpectedItems{
    // TODO: })
}
