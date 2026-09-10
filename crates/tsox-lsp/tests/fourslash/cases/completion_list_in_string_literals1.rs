use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_string_literals1() {
    let content = r#""/*1*/       /*2*/\/*3*/
 /*4*/   \\/*5*/"#;
    let mut s = Session::new_for_test("completionListInStringLiterals1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
