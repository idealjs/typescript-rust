use tsox_lsp::fourslash::{self, Session};

#[test]
fn generic_with_specialized_properties2() {
    let content = r#"interface Foo<T> {
    y: Foo<number>;
    x: Foo<string>;
}
var f: Foo<string>;
var /*1*/x = f.x; 
var /*2*/y = f.y; 
var f2: Foo<number>;
var /*3*/x2 = f2.x; 
var /*4*/y2 = f2.y; "#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "var x: Foo<string>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var y: Foo<number>", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var x2: Foo<string>", "");
    fourslash::verify_quick_info_at(&mut s, "4", "var y2: Foo<number>", "");
}
