use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.InsertLine"]
#[test]
fn formatting_on_enter() {
    let content = r#"class foo { }
class bar {/**/ }
// new line here"#;
    let mut s = Session::new_for_test("formattingOnEnter", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::verify_current_file_content(&mut s, r#"class foo { }
class bar {
}
// new line here"#);
}
