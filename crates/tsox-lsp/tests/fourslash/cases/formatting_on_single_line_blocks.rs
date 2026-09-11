use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_single_line_blocks() {
    let content = r#"class C
{}
if (true)
{}"#;
    let mut s = Session::new_for_test("formattingOnSingleLineBlocks", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"class C { }
if (true) { }"#);
}
