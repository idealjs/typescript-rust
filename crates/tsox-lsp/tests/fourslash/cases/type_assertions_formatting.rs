use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn type_assertions_formatting() {
    let content = r#"( <  any   >      publisher);/*1*/
 <  any  >      3;/*2*/"#;
    let mut s = Session::new_for_test("typeAssertionsFormatting", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"(<any>publisher);"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"<any>3;"#);
}
