use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_combine_overloads_return_type() {
    let content = r#"interface A { a: number }
interface B { b: number }
declare function f(n: number): A;
declare function f(s: string): B;
f()./**/"#;
    let mut s = Session::new_for_test("completionsCombineOverloads_returnType", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["a", "b"]);
}
