use tsox_lsp::fourslash::Session;


#[test]
fn completions_recommended_switch() {
    let content = r#"enum Enu {}
declare const e: Enu;
switch (e) {
    case E/*0*/:
    case /*1*/:
}"#;
    let _s = Session::new_for_test("completionsRecommended_switch", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
