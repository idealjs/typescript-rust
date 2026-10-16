use tsox_lsp::fourslash::Session;


#[test]
fn completions_recommended_equals() {
    let content = r#"enum Enu {}
declare const e: Enu;
e === /*a*/;
e === E/*b*/"#;
    let _s = Session::new_for_test("completionsRecommended_equals", content);
    // TODO: f.VerifyCompletions(t, []string{"a", "b"}, &fourslash.CompletionsExpectedList{
}
