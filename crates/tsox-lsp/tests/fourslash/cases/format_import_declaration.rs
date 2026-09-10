use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_import_declaration() {
    let content = r#"namespace Foo {/*1*/
}/*2*/

import bar  =    Foo;/*3*/

import bar2=Foo;/*4*/"#;
    let mut s = Session::new_for_test("formatImportDeclaration", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"namespace Foo {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"import bar = Foo;"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"import bar2 = Foo;"#);
}
