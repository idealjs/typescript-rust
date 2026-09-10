use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn function_types() {
    let content = r#"// @lib: es5
// @strict: false
var f: Function;
function g() { }

class C {
    h: () => void ;
    i(): number { return 5; }
    static j = (e) => e;
    static k() { return 'hi';}
}
var l = () => void 0;
var z = new C;

f./*1*/apply(this, [1]);
g./*2*/arguments;
z.h./*3*/bind(undefined, 1, 2);
z.i./*4*/call(null)
C.j./*5*/length === 1;
typeof C.k./*6*/caller === 'function';
l./*7*/prototype = Object.prototype;"#;
    let mut s = Session::new_for_test("functionTypes", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2", "3", "4", "5", "6"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
}
