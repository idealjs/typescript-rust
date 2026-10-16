use tsox_lsp::fourslash::{self, Session};


#[test]
fn regex_error_recovery() {
    let content = r#" // test code
//var x = //**/a/;/*1*/
//x.exec("bab");
 Bug 579071: Parser no longer detects a Regex when an open bracket is inserted
verify.quickInfoIs("RegExp");
verify.not.errorExistsAfterMarker("1");"#;
    let mut s = Session::new_for_test("regexErrorRecovery", content);
    fourslash::go_to_position(&mut s, 0);
    fourslash::insert(&mut s, "(");
}
