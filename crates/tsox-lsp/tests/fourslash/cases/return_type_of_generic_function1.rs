use tsox_lsp::fourslash::{self, Session};

#[test]
fn return_type_of_generic_function1() {
    let content = r#"interface WrappedArray<T> {
    map<U>(iterator: (value: T) => U, context?: any): U[];
}
var x: WrappedArray<string>;
var /**/y = x.map(s => s.length);"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "var y: number[]", "");
}
