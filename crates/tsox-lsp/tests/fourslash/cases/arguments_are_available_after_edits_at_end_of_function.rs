use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn arguments_are_available_after_edits_at_end_of_function() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"namespace Test1 {
	class Person {
		children: string[];
		constructor(public name: string, children: string[]) {
			/**/
		}
	}
}"#;
    let mut s = Session::new_for_test("argumentsAreAvailableAfterEditsAtEndOfFunction", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "this.children = ch");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
