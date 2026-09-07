use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_jsx_texts3() {
    let content = r#"//@Filename: file.tsx
function foo() {
const bar = "Oh no";

return (
<div>"{bar}"</div>
)
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"function foo() {
    const bar = "Oh no";

    return (
        <div>"{bar}"</div>
    )
}"#,
    );
}
