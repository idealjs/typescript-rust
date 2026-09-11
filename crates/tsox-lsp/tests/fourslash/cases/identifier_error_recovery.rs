use tsox_lsp::fourslash::{self, Session};


#[test]
fn identifier_error_recovery() {
    let content = r#"var /*1*/export/*2*/;
var foo;
var /*3*/class/*4*/;
var bar;"#;
    let mut s = Session::new_for_test("identifierErrorRecovery", content);
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "3", "4")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 3);
    // TODO: f.GoToEOF(t)
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["foo", "bar"], &[]);
}
