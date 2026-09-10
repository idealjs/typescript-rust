use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn basic_class_element_keywords() {
    let content = r#"class C {
	/*a*/
}"#;
    let mut s = Session::new_for_test("basicClassElementKeywords", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
