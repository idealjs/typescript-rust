use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_extend_array() {
    let content = r#"interface Foo<T> extends Array<T> { }
var x: Foo<string>;
var /*1*/r = x[0];
interface Foo2 extends Array<string> { }
var x2: Foo2;
var /*2*/r2 = x2[0];"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "var r: string", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var r2: string", "");
}
