use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_jsx_dotted_tag_name() {
    let content = r#"//@Filename: file.tsx
const x = (
<a-b.c>
<a-b.c></a-b.c>
</a-b.c>
);"#;
    let mut s = Session::new_for_test("formatJsxDottedTagName", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"const x = (
    <a-b.c>
        <a-b.c></a-b.c>
    </a-b.c>
);"#);
}
