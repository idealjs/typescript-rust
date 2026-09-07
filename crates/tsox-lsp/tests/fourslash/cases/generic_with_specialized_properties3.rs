use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn generic_with_specialized_properties3() {
    let content = r#"interface Foo<T, U> {
    x: Foo<T, U>;
    y: Foo<U, U>;
}
var f: Foo<number, string>;
var /*1*/xx = f.x;
var /*2*/yy = f.y;
var f2: Foo<string, number>;
var /*3*/x2 = f2.x;
var /*4*/y2 = f2.y;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var xx: Foo<number, string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var yy: Foo<string, string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var x2: Foo<string, number>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "var y2: Foo<number, number>", "")
}
