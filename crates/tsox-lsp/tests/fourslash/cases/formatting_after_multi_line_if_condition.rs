use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.Insert"]
#[test]
fn formatting_after_multi_line_if_condition() {
    let content = r#" var foo;
 if (foo &&
     foo) {
/*comment*/     // This is a comment
     foo.toString();
 /**/"#;
    let mut s = Session::new_for_test("formattingAfterMultiLineIfCondition", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("Insert"); // f.Insert(t, "}")
}
