use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_type_argument_on_new_line() {
    let content = r#"const genericObject = new GenericObject<
  /*1*/{}
>();
const genericObject2 = new GenericObject2<
  /*2*/{},
  /*3*/{}
>();"#;
    let mut s = Session::new_for_test("formatTypeArgumentOnNewLine", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"    {}"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    {},"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"    {}"#);
}
