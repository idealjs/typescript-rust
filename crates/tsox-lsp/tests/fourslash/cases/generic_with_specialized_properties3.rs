use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "1", "var xx: Foo<number, string>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var yy: Foo<string, string>", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var x2: Foo<string, number>", "");
    fourslash::verify_quick_info_at(&mut s, "4", "var y2: Foo<number, number>", "");
}
