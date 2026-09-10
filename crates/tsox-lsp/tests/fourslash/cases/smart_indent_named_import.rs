use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn smart_indent_named_import() {
    let content = r#"import {/*0*/
    numbers as bn,/*1*/
    list/*2*/
} from '@bykov/basics';/*3*/"#;
    let mut s = Session::new_for_test("smartIndentNamedImport", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "0");
    fourslash::verify_current_line_content(&mut s, r#"import {"#);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"    numbers as bn,"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    list"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"} from '@bykov/basics';"#);
}
