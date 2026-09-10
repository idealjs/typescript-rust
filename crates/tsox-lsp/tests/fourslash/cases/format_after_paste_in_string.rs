use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_after_paste_in_string() {
    let content = r#"/*2*/const x = f('aa/*1*/a').x()"#;
    let mut s = Session::new_for_test("formatAfterPasteInString", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("Paste"); // f.Paste(t, "bb")
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"const x = f('aabba').x()"#);
}
