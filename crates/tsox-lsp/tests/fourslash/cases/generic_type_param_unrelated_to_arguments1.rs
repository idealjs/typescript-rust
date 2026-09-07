use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "1", "var f1: Foo<number>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var f2: Foo<number>", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var f3: any", "");
    fourslash::verify_quick_info_at(&mut s, "4", "var f4: Foo<number>", "");
    fourslash::verify_quick_info_at(&mut s, "5", "var f5: any", "");
    fourslash::verify_quick_info_at(&mut s, "6", "var f6: Foo<number>", "");
}
