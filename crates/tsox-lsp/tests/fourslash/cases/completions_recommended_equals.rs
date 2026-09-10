use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_recommended_equals() {
    let content = r#"enum Enu {}
declare const e: Enu;
e === /*a*/;
e === E/*b*/"#;
    let mut s = Session::new_for_test("completionsRecommended_equals", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"a", "b"}, &fourslash.CompletionsExpectedList{
}
