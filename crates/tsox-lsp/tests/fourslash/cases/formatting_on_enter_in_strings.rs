use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.InsertLine"]
#[test]
fn formatting_on_enter_in_strings() {
    let content = r#"var x = /*1*/"unclosed string literal\/*2*/"#;
    let mut s = Session::new_for_test("formattingOnEnterInStrings", content);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, "var x = \"unclosed string literal\\");
}
