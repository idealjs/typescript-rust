use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNumberOfErrorsInCurrentFile"]
#[test]
fn insert_return_statement_in_duplicate_identifier_function() {
    let content = r#"// @strict: true
class foo { };
function foo() { /**/ }"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 2)
    fourslash::insert(&mut s, "return null;");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 2)
}
