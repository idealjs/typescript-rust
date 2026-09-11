use tsox_lsp::fourslash::{self, Session};


#[test]
fn unreachable_statement_node_reuse() {
    let content = r#"function test() {
	return/*a*/abc();
	return;
}
function abc() { }"#;
    let mut s = Session::new_for_test("unreachableStatementNodeReuse", content);
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
    fourslash::go_to_marker(&mut s, "a");
    fourslash::insert(&mut s, " ");
    fourslash::verify_no_errors(&mut s, );
}
