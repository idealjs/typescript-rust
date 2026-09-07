use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn generic_call_signatures_in_non_generic_types1() {
    let content = r#"interface WrappedObject<T> { }
interface WrappedArray<T> { }
interface Underscore {
    <T>(list: T[]): WrappedArray<T>;
    <T>(obj: T): WrappedObject<T>;
}
var _: Underscore;
var a: number[];
var /**/b = _(a); "#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var b: WrappedArray<number>", "")
}
