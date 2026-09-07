use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn optional_property_formatting() {
    let content = r#"export class C extends Error {
    message: string;
    data? = {};
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"export class C extends Error {
    message: string;
    data? = {};
}"#,
    );
}
