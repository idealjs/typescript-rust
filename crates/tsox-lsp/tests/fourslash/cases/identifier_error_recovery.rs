use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.GoToEOF"]
#[test]
fn identifier_error_recovery() {
    let content = r#"var /*1*/export/*2*/;
var foo;
var /*3*/class/*4*/;
var bar;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "3", "4")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 3);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
