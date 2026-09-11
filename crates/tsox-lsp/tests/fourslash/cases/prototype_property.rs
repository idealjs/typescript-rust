use tsox_lsp::fourslash::{self, Session};


#[test]
fn prototype_property() {
    let content = r#"class A {}
A./*1*/prototype;
A./*2*/"#;
    let mut s = Session::new_for_test("prototypeProperty", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) A.prototype: A", "");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
