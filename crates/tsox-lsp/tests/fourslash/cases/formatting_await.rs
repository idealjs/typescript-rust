use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_await() {
    let content = r#"async function f() {
    for          await (const x of g()) {
        console.log(x);
    }
}"#;
    let mut s = Session::new_for_test("formattingAwait", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"async function f() {
    for await (const x of g()) {
        console.log(x);
    }
}"#);
}
