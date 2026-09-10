use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn class_member_completion_keeps_name_fallback() {
    let content = r#"class B {
	constructor(public value: string) {}
}
class C extends B {
	/*a*/
}"#;
    let mut s = Session::new_for_test("classMemberCompletionKeepsNameFallback", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn implement_class_fix_does_not_add_invalid_override() {
    let content = r#"// @noImplicitOverride: true
class B {
    method() {}
}
class C implements B {[| |]}"#;
    let mut s = Session::new_for_test("implementClassFixDoesNotAddInvalidOverride", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
