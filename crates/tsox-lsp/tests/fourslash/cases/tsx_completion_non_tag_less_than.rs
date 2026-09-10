use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn tsx_completion_non_tag_less_than() {
    let content = r#"// @lib: es5
// @Filename: /a.tsx
var x: Array<numb/*a*/;
[].map<numb/*b*/;
1 < Infini/*c*/;"#;
    let mut s = Session::new_for_test("tsxCompletionNonTagLessThan", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"a", "b"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "c", &fourslash.CompletionsExpectedList{
}
