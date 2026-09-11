use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_unused_label_no_suggestion_if_disabled() {
    let content = r#"// @allowUnusedLabels: true
[|foo|]: while (true) {}"#;
    let mut s = Session::new_for_test("codeFixUnusedLabel_noSuggestionIfDisabled", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}
