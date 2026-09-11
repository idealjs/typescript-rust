use tsox_lsp::fourslash::{self, Session};


#[test]
fn space_before_and_after_binary_operators() {
    let content = r#"let i = 0;
/*1*/(i++,i++);
/*2*/(i++,++i);
/*3*/(1,2);
/*4*/(i++,2);
/*5*/(i++,i++,++i,i--,2);
let s = 'foo';
/*6*/for (var i = 0,ii = 2; i < s.length; ii++,i++) {
}"#;
    let mut s = Session::new_for_test("spaceBeforeAndAfterBinaryOperators", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"(i++, i++);"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"(i++, ++i);"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"(1, 2);"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"(i++, 2);"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"(i++, i++, ++i, i--, 2);"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"for (var i = 0, ii = 2; i < s.length; ii++, i++) {"#);
    // TODO: }
}
