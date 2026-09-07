use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatSelection"]
#[test]
fn format_selection_jsx_with_binary_expression() {
    let content = r#"//@Filename: file.tsx
function TestWidget() {
    const test = true;
    return (
        <div>
            {test &&
                <div>
 /*1*/                <div>some text</div>/*2*/
                    <div>some text</div>
                    <div>some text</div>
                </div>
            }
            <div>some text</div>
        </div>
    );
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatSelection"); // f.FormatSelection(t, "1", "2")
    fourslash::verify_current_file_content(
        &mut s,
        r#"function TestWidget() {
    const test = true;
    return (
        <div>
            {test &&
                <div>
                    <div>some text</div>
                    <div>some text</div>
                    <div>some text</div>
                </div>
            }
            <div>some text</div>
        </div>
    );
}"#,
    );
}
