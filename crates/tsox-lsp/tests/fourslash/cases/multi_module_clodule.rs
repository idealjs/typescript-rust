use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn multi_module_clodule() {
    let content = r#"// @lib: es5
class C {
    constructor(x: number) { }
    foo() { }
    bar() { }
    static boo() { }
}

namespace C {
    export var x = 1;
    var y = 2;
}
namespace C {
    export function foo() { }
    function baz() { return ''; }
}

var c = new C/*1*/(C./*2*/x);
c./*3*/foo = C./*4*/foo;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"2", "4"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::verify_no_errors(&mut s);
}
