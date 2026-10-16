use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_in_template_literal_parts1() {
    let content = r#"// @lib: es5
/*0*/`  $ { ${/*1*/ 10/*2*/ + 1.1/*3*/ /*4*/} 12312`/*5*/

/*6*/`asdasd${/*7*/ 2 + 1.1 /*8*/} 12312 {"#;
    let _s = Session::new_for_test("completionListInTemplateLiteralParts1", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "7"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2", "3", "4", "5", "6", "8"}, &fourslash.CompletionsExpectedList{
    // TODO: }
}
