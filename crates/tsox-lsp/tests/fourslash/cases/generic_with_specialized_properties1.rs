use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn generic_with_specialized_properties1() {
    let content = r#"interface Foo<T> {
    x: Foo<string>;
    y: Foo<number>;
}
var f: Foo<number>;
var /*1*/xx = f.x;
var /*2*/yy = f.y;
var f2: Foo<string>;
var /*3*/x2 = f2.x;
var /*4*/y2 = f2.y;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var xx: Foo<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var yy: Foo<number>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var x2: Foo<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "var y2: Foo<number>", "")
}
