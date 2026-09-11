use tsox_lsp::fourslash::{self, Session};


#[test]
fn contextual_typing_from_type_assertion1() {
    let content = r#"var f3 = <(x: string) => string> function (/**/x) { return x.toLowerCase(); };"#;
    let mut s = Session::new_for_test("contextualTypingFromTypeAssertion1", content);
    fourslash::verify_quick_info_at(&mut s, "", "(parameter) x: string", "");
}
