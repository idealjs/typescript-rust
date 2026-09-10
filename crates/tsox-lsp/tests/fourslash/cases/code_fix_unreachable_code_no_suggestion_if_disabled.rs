use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn code_fix_unreachable_code_no_suggestion_if_disabled() {
    let content = r#"// @allowUnreachableCode: true
if (false) [|0;|]"#;
    let mut s = Session::new_for_test("codeFixUnreachableCode_noSuggestionIfDisabled", content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, nil)
}
