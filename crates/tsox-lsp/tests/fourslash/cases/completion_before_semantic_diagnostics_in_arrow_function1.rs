use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_before_semantic_diagnostics_in_arrow_function1() {
    let content = r#"var f4 = <T>(x: T/**/ ) => {
}"#;
    let mut s = Session::new_for_test("completionBeforeSemanticDiagnosticsInArrowFunction1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.Backspace(t, 1)
    fourslash::insert(&mut s, "A");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
