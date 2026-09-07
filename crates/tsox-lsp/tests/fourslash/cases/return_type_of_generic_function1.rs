use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn return_type_of_generic_function1() {
    let content = r#"interface WrappedArray<T> {
    map<U>(iterator: (value: T) => U, context?: any): U[];
}
var x: WrappedArray<string>;
var /**/y = x.map(s => s.length);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var y: number[]", "")
}
