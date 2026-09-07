use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_on_circular_types() {
    let content = r#"interface A { (): B; };
declare var a: A;
var xx = a();

interface B { (): C; };
declare var b: B;
var yy = b();

interface C { (): A; };
declare var c: C;
var zz = c();

x/*B*/x = y/*C*/y;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "B", "var xx: B", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "C", "var yy: C", "")
}
