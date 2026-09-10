use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn completion_entry_for_shorthand_property_assignment() {
    let content = r#"var person: {name:string; id:number} = {n/**/"#;
    let mut s = Session::new_for_test("completionEntryForShorthandPropertyAssignment", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: }
}
