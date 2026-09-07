use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_document_with_js_doc() {
    let content = r#"/**
 * JSDoc for things
 */
function f() {
    /** more
        jsdoc */
    var t;
    /**
     * multiline
     */
    var multiline;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"/**
 * JSDoc for things
 */
function f() {
    /** more
        jsdoc */
    var t;
    /**
     * multiline
     */
    var multiline;
}"#,
    );
}
