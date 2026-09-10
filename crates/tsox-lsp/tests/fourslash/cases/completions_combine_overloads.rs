use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_combine_overloads() {
    let content = r#"interface A { a: number }
interface B { b: number }
declare function f(a: A): void;
declare function f(b: B): void;
f({ /**/ });"#;
    let mut s = Session::new_for_test("completionsCombineOverloads", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["a", "b"]);
}
