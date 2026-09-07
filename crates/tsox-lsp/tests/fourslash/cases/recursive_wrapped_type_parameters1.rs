use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "1", "var yy: I<I<I<I<I<I<number>>>>>>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var aa: number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var bb: I<number>", "");
    fourslash::verify_quick_info_at(&mut s, "4", "var cc: I<I<number>>", "");
    fourslash::verify_quick_info_at(&mut s, "5", "var dd: I<number>", "");
    fourslash::verify_quick_info_at(&mut s, "6", "var ee: I<I<number>>", "");
    fourslash::verify_quick_info_at(&mut s, "7", "var ff: I<I<I<number>>>", "");
}
