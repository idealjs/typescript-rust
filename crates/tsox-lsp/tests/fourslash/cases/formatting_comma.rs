use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_comma() {
    let content = r#"var x = [1 , 2];/*x*/
var y = ( 1  , 2 );/*y*/
var z1 = 1 , zz = 2;/*z1*/
var z2 = {
    x: 1 ,/*z2*/
    y: 2
};
var z3 = (
    () => { }  ,/*z3*/
    () => { }
    );
var z4 = [
    () => { } ,/*z4*/
    () => { }
];
var z5 = {
    x: () => { } ,/*z5*/
    y: () => { }
}; "#;
    let mut s = Session::new_for_test("formattingComma", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "x");
    fourslash::verify_current_line_content(&mut s, r#"var x = [1, 2];"#);
    fourslash::go_to_marker(&mut s, "y");
    fourslash::verify_current_line_content(&mut s, r#"var y = (1, 2);"#);
    fourslash::go_to_marker(&mut s, "z1");
    fourslash::verify_current_line_content(&mut s, r#"var z1 = 1, zz = 2;"#);
    fourslash::go_to_marker(&mut s, "z2");
    fourslash::verify_current_line_content(&mut s, r#"    x: 1,"#);
    fourslash::go_to_marker(&mut s, "z3");
    fourslash::verify_current_line_content(&mut s, r#"    () => { },"#);
    fourslash::go_to_marker(&mut s, "z4");
    fourslash::verify_current_line_content(&mut s, r#"    () => { },"#);
    fourslash::go_to_marker(&mut s, "z5");
    fourslash::verify_current_line_content(&mut s, r#"    x: () => { },"#);
}
