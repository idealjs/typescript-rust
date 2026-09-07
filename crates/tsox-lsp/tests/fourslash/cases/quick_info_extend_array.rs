use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_extend_array() {
    let content = r#"interface Foo<T> extends Array<T> { }
var x: Foo<string>;
var /*1*/r = x[0];
interface Foo2 extends Array<string> { }
var x2: Foo2;
var /*2*/r2 = x2[0];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var r: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var r2: string", "")
}
