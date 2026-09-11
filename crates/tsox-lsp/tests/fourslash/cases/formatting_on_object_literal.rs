use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_object_literal() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"var x = /*1*/{foo:/*2*/ 1,
bar: "tt",/*3*/
boo: /*4*/1 + 5}/*5*/;

var x2 = /*6*/{foo/*7*/: 1,
bar: /*8*/"tt",boo:1+5}/*9*/;

function Foo() {/*10*/
var typeICalc = {/*11*/
clear: {/*12*/
"()": [1, 2, 3]/*13*/
}/*14*/
}/*15*/
}/*16*/

// Rule for object literal members for the "value" of the memebr to follow the indent/*17*/
// of the member, i.e. the relative position of the value is maintained when the member/*18*/
// is indented./*19*/
var x2 = {/*20*/
  foo:/*21*/
3,/*22*/
          'bar':/*23*/
                    { a: 1, b : 2}/*24*/
};/*25*/

var x={    };/*26*/
var y = {};/*27*/"#;
    let mut s = Session::new_for_test("formattingOnObjectLiteral", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"var x = {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    foo: 1,"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"    bar: "tt","#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"    boo: 1 + 5"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"};"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"var x2 = {"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"    foo: 1,"#);
    fourslash::go_to_marker(&mut s, "8");
    fourslash::verify_current_line_content(&mut s, r#"    bar: "tt", boo: 1 + 5"#);
    fourslash::go_to_marker(&mut s, "9");
    fourslash::verify_current_line_content(&mut s, r#"};"#);
    fourslash::go_to_marker(&mut s, "10");
    fourslash::verify_current_line_content(&mut s, r#"function Foo() {"#);
    fourslash::go_to_marker(&mut s, "11");
    fourslash::verify_current_line_content(&mut s, r#"    var typeICalc = {"#);
    fourslash::go_to_marker(&mut s, "12");
    fourslash::verify_current_line_content(&mut s, r#"        clear: {"#);
    fourslash::go_to_marker(&mut s, "13");
    fourslash::verify_current_line_content(&mut s, r#"            "()": [1, 2, 3]"#);
    fourslash::go_to_marker(&mut s, "14");
    fourslash::verify_current_line_content(&mut s, r#"        }"#);
    fourslash::go_to_marker(&mut s, "15");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "16");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "17");
    fourslash::verify_current_line_content(&mut s, r#"// Rule for object literal members for the "value" of the memebr to follow the indent"#);
    fourslash::go_to_marker(&mut s, "18");
    fourslash::verify_current_line_content(&mut s, r#"// of the member, i.e. the relative position of the value is maintained when the member"#);
    fourslash::go_to_marker(&mut s, "19");
    fourslash::verify_current_line_content(&mut s, r#"// is indented."#);
    fourslash::go_to_marker(&mut s, "20");
    fourslash::verify_current_line_content(&mut s, r#"var x2 = {"#);
    fourslash::go_to_marker(&mut s, "21");
    fourslash::verify_current_line_content(&mut s, r#"    foo:"#);
    fourslash::go_to_marker(&mut s, "22");
    fourslash::verify_current_line_content(&mut s, r#"        3,"#);
    fourslash::go_to_marker(&mut s, "23");
    fourslash::verify_current_line_content(&mut s, r#"    'bar':"#);
    fourslash::go_to_marker(&mut s, "24");
    fourslash::verify_current_line_content(&mut s, r#"        { a: 1, b: 2 }"#);
    fourslash::go_to_marker(&mut s, "25");
    fourslash::verify_current_line_content(&mut s, r#"};"#);
    fourslash::go_to_marker(&mut s, "26");
    fourslash::verify_current_line_content(&mut s, r#"var x = {};"#);
    fourslash::go_to_marker(&mut s, "27");
    fourslash::verify_current_line_content(&mut s, r#"var y = {};"#);
}
