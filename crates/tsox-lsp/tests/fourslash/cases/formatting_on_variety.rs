use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_on_variety() {
    let content = r#"function f(a,b,c,d){/*1*/
for(var i=0;i<10;i++){/*2*/
var a=0;/*3*/
var b=a+a+a*a%a/2-1;/*4*/
b+=a;/*5*/
++b;/*6*/
f(a,b,c,d);/*7*/
if(1===1){/*8*/
var m=function(e,f){/*9*/
return e^f;/*10*/
}/*11*/
}/*12*/
}/*13*/
}/*14*/

for (var i = 0   ; i < this.foo(); i++) {/*15*/
}/*16*/"#;
    let mut s = Session::new_for_test("formattingOnVariety", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"function f(a, b, c, d) {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    for (var i = 0; i < 10; i++) {"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"        var a = 0;"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"        var b = a + a + a * a % a / 2 - 1;"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"        b += a;"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"        ++b;"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"        f(a, b, c, d);"#);
    fourslash::go_to_marker(&mut s, "8");
    fourslash::verify_current_line_content(&mut s, r#"        if (1 === 1) {"#);
    fourslash::go_to_marker(&mut s, "9");
    fourslash::verify_current_line_content(&mut s, r#"            var m = function(e, f) {"#);
    fourslash::go_to_marker(&mut s, "10");
    fourslash::verify_current_line_content(&mut s, r#"                return e ^ f;"#);
    fourslash::go_to_marker(&mut s, "11");
    fourslash::verify_current_line_content(&mut s, r#"            }"#);
    fourslash::go_to_marker(&mut s, "12");
    fourslash::verify_current_line_content(&mut s, r#"        }"#);
    fourslash::go_to_marker(&mut s, "13");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "14");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "15");
    fourslash::verify_current_line_content(&mut s, r#"for (var i = 0; i < this.foo(); i++) {"#);
    fourslash::go_to_marker(&mut s, "16");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
}
