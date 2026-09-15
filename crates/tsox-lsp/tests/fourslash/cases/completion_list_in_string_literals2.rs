use tsox_lsp::fourslash::Session;


#[test]
fn completion_list_in_string_literals2() {
    let content = r#""/*1*/       /*2*/\/*3*/
 /*4*/   \\\/*5*/
 /*6*/"#;
    let _s = Session::new_for_test("completionListInStringLiterals2", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
