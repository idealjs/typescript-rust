use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_tsx_multiline_attribute_string() {
    let content = r#"// @Filename: foo.tsx
(
    <input
        value="x
        x"
    />
);"#;
    let mut s = Session::new_for_test("formatTsxMultilineAttributeString", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"(
    <input
        value="x
        x"
    />
);"#);
}
