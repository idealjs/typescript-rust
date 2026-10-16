use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_unreachable_code_no_suggestion_if_disabled() {
    let content = r#"// @allowUnreachableCode: true
if (false) [|0;|]"#;
    let _s = Session::new_for_test("codeFixUnreachableCode_noSuggestionIfDisabled", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}
