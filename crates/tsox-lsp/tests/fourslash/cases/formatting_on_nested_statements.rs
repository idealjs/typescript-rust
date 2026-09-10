use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatSelection"]
#[test]
fn formatting_on_nested_statements() {
    let content = r#"{
/*1*/{
/*3*/test
}/*2*/
}"#;
    let mut s = Session::new_for_test("formattingOnNestedStatements", content);
    fourslash::unsupported("FormatSelection"); // f.FormatSelection(t, "1", "2")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"    {"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"        test"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
}
