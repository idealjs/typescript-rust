use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn prototype_property() {
    let content = r#"class A {}
A./*1*/prototype;
A./*2*/"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) A.prototype: A", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
