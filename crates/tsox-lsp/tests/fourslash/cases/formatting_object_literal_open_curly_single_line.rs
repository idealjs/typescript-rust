use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_object_literal_open_curly_single_line() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"
let obj1 =
{ x: 10 };

let obj2 =
    // leading trivia
{ y: 10 };
"#;
    let mut s = Session::new_for_test("formattingObjectLiteralOpenCurlySingleLine", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"
let obj1 =
    { x: 10 };

let obj2 =
    // leading trivia
    { y: 10 };
"#);
}
