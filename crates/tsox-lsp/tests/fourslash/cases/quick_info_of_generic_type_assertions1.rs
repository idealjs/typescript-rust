use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_of_generic_type_assertions1() {
    let content = r#"function f<T>(x: T): T { return null; }
var /*1*/r = <T>(x: T) => x;
var /*2*/r2 = < <T>(x: T) => T>f;
var a;
var /*3*/r3 = < <T>(x: <A>(y: A) => A) => T>a;"#;
    let mut s = Session::new_for_test("quickInfoOfGenericTypeAssertions1", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var r: <T>(x: T) => T", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var r2: <T>(x: T) => T", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var r3: <T>(x: <A>(y: A) => A) => T", "");
}
