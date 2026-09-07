use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_in_tsx_files() {
    let content = r#"//@Filename: file.tsx
interface I<T1, T2> {
    next: I</* */
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
}
