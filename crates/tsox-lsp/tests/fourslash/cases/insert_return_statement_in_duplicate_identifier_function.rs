use tsox_lsp::fourslash::{self, Session};


#[test]
fn insert_return_statement_in_duplicate_identifier_function() {
    let content = r#"// @strict: true
class foo { };
function foo() { /**/ }"#;
    let mut s = Session::new_for_test("insertReturnStatementInDuplicateIdentifierFunction", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 2);
    fourslash::insert(&mut s, "return null;");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 2);
}
