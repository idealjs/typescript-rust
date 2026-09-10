use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn indentation_in_jsx3() {
    let content = r#"//@Filename: file.tsx
function foo() {
   return (
        <div>
hello
goodbye
        </div>
    )
}"#;
    let mut s = Session::new_for_test("indentationInJsx3", content);
    fourslash::verify_current_file_content(&mut s, r#"function foo() {
   return (
        <div>
hello
goodbye
        </div>
    )
}"#);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"function foo() {
    return (
        <div>
            hello
            goodbye
        </div>
    )
}"#);
}
