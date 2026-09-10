use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_type_parameter_of_type_alias1() {
    let content = r#"type List1</*0*/
type List2</*1*/T> = T[];
type List4<T> = /*2*/T[];
type List3<T1> = /*3*/;"#;
    let mut s = Session::new_for_test("completionListInTypeParameterOfTypeAlias1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"0", "1"}, nil)
    fourslash::verify_completions_include_exclude_at(&mut s, Some("2"), &["T"], &[]);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
