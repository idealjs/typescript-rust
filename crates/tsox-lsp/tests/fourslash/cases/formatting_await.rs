use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_await() {
    let content = r#"async function f() {
    for          await (const x of g()) {
        console.log(x);
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"async function f() {
    for await (const x of g()) {
        console.log(x);
    }
}"#,
    );
}
