use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn clodule_as_base_class() {
    let content = r#"// @lib: es5
// @strict: false
class A {
    constructor(x: number) { }
    foo() { }
    static bar() { }
}

namespace A {
    export var x = 1;
    export function baz() { }
}

class D extends A {
    constructor() {
        super(1);
    }
    foo2() { }
    static bar2() { }
}

var d: D;
d./*1*/
D./*2*/"#;
    let mut s = Session::new_for_test("cloduleAsBaseClass", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["foo", "foo2"]);
    fourslash::insert(&mut s, "foo()");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "bar()");
    fourslash::verify_no_errors(&mut s, );
}
