use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_after_async_in_object_literal() {
    let content = r#"const x: { m(): Promise<void> } = { async /**/ };"#;
    let mut s = Session::new_for_test("completionsAfterAsyncInObjectLiteral", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["m"]);
}
