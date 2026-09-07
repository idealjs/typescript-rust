use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_jsx_dotted_tag_name() {
    let content = r#"//@Filename: file.tsx
const x = (
<a-b.c>
<a-b.c></a-b.c>
</a-b.c>
);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"const x = (
    <a-b.c>
        <a-b.c></a-b.c>
    </a-b.c>
);"#,
    );
}
