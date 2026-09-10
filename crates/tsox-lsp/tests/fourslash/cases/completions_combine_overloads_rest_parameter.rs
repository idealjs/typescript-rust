use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_combine_overloads_rest_parameter() {
    let content = r#"interface A { a: number }
interface B { b: number }
interface C { c: number }
declare function f(a: A): void;
declare function f(...bs: B[]): void;
declare function f(...cs: C[]): void;
f({ /*1*/ });
f({ a: 1 }, { /*2*/ });"#;
    let mut s = Session::new_for_test("completionsCombineOverloads_restParameter", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["a", "b", "c"]);
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["b", "c"]);
}
