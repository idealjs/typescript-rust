use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("extendsTArray", content);
    fourslash::verify_quick_info_at(&mut s, "", "var y: Date[]", "");
    fourslash::verify_no_errors(&mut s, );
}
