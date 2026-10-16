use tsox_lsp::fourslash::{self, Session};


#[test]
fn function_type_predicate_formatting() {
    let content = r#"/**/function bar(a: A):     a        is       B    {}"#;
    let mut s = Session::new_for_test("functionTypePredicateFormatting", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"function bar(a: A): a is B { }"#);
}
