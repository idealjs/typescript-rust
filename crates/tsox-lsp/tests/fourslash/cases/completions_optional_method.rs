use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_optional_method() {
    let content = r#"// @strictNullChecks: true
declare const x: { m?(): void };
x./**/"#;
    let mut s = Session::new_for_test("completionsOptionalMethod", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["m?"]);
}
