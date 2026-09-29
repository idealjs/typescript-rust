use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_with_statement() {
    let content = r#"with /*1*/(foo.bar)

   {/*2*/

     }/*3*/

with (bar.blah)/*4*/
{/*5*/
}/*6*/"#;
    let mut s = Session::new_for_test("formatWithStatement", content);
    // TODO: opts227 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("place_open_brace_on_new_line_for_control_blocks", "false")]);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"with (foo.bar) {"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"with (bar.blah) {"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    // TODO: opts565 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("place_open_brace_on_new_line_for_control_blocks", "true")]);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"with (foo.bar)"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"{"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"with (bar.blah)"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"{"#);
    // TODO: }
}
