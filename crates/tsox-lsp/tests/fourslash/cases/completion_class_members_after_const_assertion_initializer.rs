use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn class_members_after_const_assertion_initializer() {
    let content = r#"
interface A {
	a: number
	def: string
}

class B implements A {
	a = 1 as const
	/**/
}
"#;
    let mut s = Session::new_for_test("classMembersAfterConstAssertionInitializer", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
