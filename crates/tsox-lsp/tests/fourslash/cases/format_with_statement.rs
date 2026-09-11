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
    // TODO: opts227.FormatCodeSettings.PlaceOpenBraceOnNewLineForControlBlocks = core.TSFalse
    // TODO: f.Configure(t, opts227)
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"with (foo.bar) {"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"with (bar.blah) {"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    // TODO: opts565 := f.GetOptions()
    // TODO: opts565.FormatCodeSettings.PlaceOpenBraceOnNewLineForControlBlocks = core.TSTrue
    // TODO: f.Configure(t, opts565)
    // TODO: f.FormatDocument(t, "")
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
