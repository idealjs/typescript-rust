use tsox_lsp::fourslash::{self, Session};


#[test]
fn duplicate_function_implementation() {
    let content = r#"interface IFoo<T> {
    foo<T>(): T;
}
function foo<string>(/**/): string { return null; }
function foo<T>(x: T): T { return null; }"#;
    let mut s = Session::new_for_test("duplicateFunctionImplementation", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "x: string");
}
