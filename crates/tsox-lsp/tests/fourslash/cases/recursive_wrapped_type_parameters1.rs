use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn recursive_wrapped_type_parameters1() {
    let content = r#"interface I<T> {
	a: T;
	b: I<T>;
	c: I<I<T>>;
}
var x: I<number>;
var y/*1*/y = x.c.c.c.c.c.b;
var a/*2*/a = x.a;
var b/*3*/b = x.b;
var c/*4*/c = x.c;
var d/*5*/d = x.c.a;
var e/*6*/e = x.c.b;
var f/*7*/f = x.c.c; "#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var yy: I<I<I<I<I<I<number>>>>>>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var aa: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var bb: I<number>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "var cc: I<I<number>>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "var dd: I<number>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "var ee: I<I<number>>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "7", "var ff: I<I<I<number>>>", "")
}
