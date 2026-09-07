use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn extends_t_array() {
    let content = r#"// @strict: false
interface I1<T> {
    (a: T): T;
}
interface I2<T> extends I1<T[]> {
    b: T;
}
var x: I2<Date>;
var /**/y = x(undefined); // Typeof y should be Date[]
y.length;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var y: Date[]", "")
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
