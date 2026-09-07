use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn generic_type_param_unrelated_to_arguments1() {
    let content = r#"interface Foo<T> {
    new (x: number): Foo<T>;
}
declare var f/*1*/1: Foo<number>;
var f/*2*/2: Foo<number>;
var f/*3*/3 = new Foo(3);
var f/*4*/4: Foo<number> = new Foo(3);
var f/*5*/5 = new Foo<number>(3);
var f/*6*/6: Foo<number> = new Foo<number>(3);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var f1: Foo<number>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var f2: Foo<number>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var f3: any", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "var f4: Foo<number>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "var f5: any", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "var f6: Foo<number>", "")
}
