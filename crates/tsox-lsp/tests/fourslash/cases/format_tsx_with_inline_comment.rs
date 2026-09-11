use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_tsx_with_inline_comment() {
    let content = r#"// @Filename: foo.tsx
const a = <div>
    // <a />
</div>"#;
    let mut s = Session::new_for_test("formatTSXWithInlineComment", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"const a = <div>
    // <a />
</div>"#);
}
