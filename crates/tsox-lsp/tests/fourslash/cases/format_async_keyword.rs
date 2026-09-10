use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_async_keyword() {
    let content = r#"/*1*/let x = async         () => 1;
/*2*/let y = async() => 1;
/*3*/let z = async    function   () { return 1; };"#;
    let mut s = Session::new_for_test("formatAsyncKeyword", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"let x = async () => 1;"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"let y = async () => 1;"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"let z = async function() { return 1; };"#);
}
