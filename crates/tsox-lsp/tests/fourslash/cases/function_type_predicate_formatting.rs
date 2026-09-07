use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn function_type_predicate_formatting() {
    let content = r#"/**/function bar(a: A):     a        is       B    {}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_line_content(&mut s, r#"function bar(a: A): a is B { }"#);
}
