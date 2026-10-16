use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_function() {
    let content = r#"/**/function foo() { return "hi"; }"#;
    let mut s = Session::new_for_test("quickInfoFunction", content);
    fourslash::verify_quick_info_at(&mut s, "", "function foo(): string", "");
}
