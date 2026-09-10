use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_non_null_assertion_operator() {
    let content = r#"/*1*/ 'bar' ! ;
/*2*/ ( 'bar' ) ! ;
/*3*/ 'bar' [ 1 ] ! ;
/*4*/ var  bar  =  'bar' . foo ! ;
/*5*/ var  foo  =  bar ! ;"#;
    let mut s = Session::new_for_test("formattingNonNullAssertionOperator", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"'bar'!;"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"('bar')!;"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"'bar'[1]!;"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"var bar = 'bar'.foo!;"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"var foo = bar!;"#);
}
