use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn unreachable_statement_node_reuse() {
    let content = r#"function test() {
	return/*a*/abc();
	return;
}
function abc() { }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
    fourslash::go_to_marker(&mut s, "a");
    fourslash::insert(&mut s, " ");
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
