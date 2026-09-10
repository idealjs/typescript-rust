use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.InsertLine"]
#[test]
fn tabbing_after_newline_inserted_before_while() {
    let content = r#"function foo() {
    /**/while (true) { }
}"#;
    let mut s = Session::new_for_test("tabbingAfterNewlineInsertedBeforeWhile", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::verify_current_line_content(&mut s, r#"    while (true) { }"#);
}
