use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn function_formatting() {
    let content = r#"var foo = foo(function () {
    /**/function foo  ()  {}}    );"#;
    let mut s = Session::new_for_test("functionFormatting", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"    function foo() { }"#);
}
