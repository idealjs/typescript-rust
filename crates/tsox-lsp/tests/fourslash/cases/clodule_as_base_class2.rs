use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn clodule_as_base_class2() {
    let content = r#"// @module: commonjs
// @strict: false
// @Filename: cloduleAsBaseClass2_0.ts
class A {
    constructor(x: number) { }
    foo() { }
    static bar() { }
}

namespace A {
    export var x = 1;
    export function baz() { }
}

export = A;
// @Filename: cloduleAsBaseClass2_1.ts
import B = require('./cloduleAsBaseClass2_0');
class D extends B {
    constructor() {
        super(1);
    }
    foo2() { }
    static bar2() { }
}

var d: D;
d./*1*/
D./*2*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "foo()");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "bar()");
    fourslash::verify_no_errors(&mut s);
}
