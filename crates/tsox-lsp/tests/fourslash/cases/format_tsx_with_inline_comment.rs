use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_tsx_with_inline_comment() {
    let content = r#"// @Filename: foo.tsx
const a = <div>
    // <a />
</div>"#;
    let mut s = Session::new_for_test("formatTSXWithInlineComment", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"const a = <div>
    // <a />
</div>"#);
}
