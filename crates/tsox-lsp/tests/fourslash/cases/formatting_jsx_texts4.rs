use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_jsx_texts4() {
    let content = r#"//@Filename: file.tsx
function foo() {
const a = <ns: foobar   x : test1   x :test2="string"  x:test3={true?1:0}  />;

return a;
}"#;
    let mut s = Session::new_for_test("formattingJsxTexts4", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"function foo() {
    const a = <ns:foobar x:test1 x:test2="string" x:test3={true ? 1 : 0} />;

    return a;
}"#);
}
