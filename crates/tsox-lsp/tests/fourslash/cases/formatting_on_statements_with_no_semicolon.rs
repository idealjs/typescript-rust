use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_statements_with_no_semicolon() {
    let content = r#"/*1*/do
     { var a/*2*/
/*3*/}   while (1)
/*4*/function f() {
/*5*/    var s = 1
/*6*/            }
/*7*/switch (t) {
/*8*/    case 1:
/*9*/{
/*10*/test
/*11*/}
/*12*/}
/*13*/do{do{do{}while(a!==b)}while(a!==b)}while(a!==b)
/*14*/do{
/*15*/do{
/*16*/do{
/*17*/}while(a!==b)
/*18*/}while(a!==b)
/*19*/}while(a!==b)
/*20*/for(var i=0;i<10;i++){
/*21*/for(var j=0;j<10;j++){
/*22*/j-=i
/*23*/}/*24*/}
/*25*/function foo() {
/*26*/try {
/*27*/x+=2
/*28*/}
/*29*/catch( e){
/*30*/x+=2
/*31*/}finally {
/*32*/x+=2
/*33*/}
/*34*/}
/*35*/do     { var a }   while (1)
    foo(function (file) {/*49*/
        return 0/*50*/
    }).then(function (doc) {/*51*/
        return 1/*52*/
    });/*53*/
/*54*/if(1)
/*55*/if(1)
/*56*/x++
/*57*/else
/*58*/if(1)
/*59*/x+=2
/*60*/else
/*61*/x+=2



/*62*/;
         do do do do/*63*/
                test;/*64*/
            while (0)/*65*/
         while (0)/*66*/
            while (0)/*67*/
         while (0)/*68*/"#;
    let mut s = Session::new_for_test("formattingOnStatementsWithNoSemicolon", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"do {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    var a"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"} while (1)"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"function f() {"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"    var s = 1"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"switch (t) {"#);
    fourslash::go_to_marker(&mut s, "8");
    fourslash::verify_current_line_content(&mut s, r#"    case 1:"#);
    fourslash::go_to_marker(&mut s, "9");
    fourslash::verify_current_line_content(&mut s, r#"        {"#);
    fourslash::go_to_marker(&mut s, "10");
    fourslash::verify_current_line_content(&mut s, r#"            test"#);
    fourslash::go_to_marker(&mut s, "11");
    fourslash::verify_current_line_content(&mut s, r#"        }"#);
    fourslash::go_to_marker(&mut s, "12");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "13");
    fourslash::verify_current_line_content(&mut s, r#"do { do { do { } while (a !== b) } while (a !== b) } while (a !== b)"#);
    fourslash::go_to_marker(&mut s, "14");
    fourslash::verify_current_line_content(&mut s, r#"do {"#);
    fourslash::go_to_marker(&mut s, "15");
    fourslash::verify_current_line_content(&mut s, r#"    do {"#);
    fourslash::go_to_marker(&mut s, "16");
    fourslash::verify_current_line_content(&mut s, r#"        do {"#);
    fourslash::go_to_marker(&mut s, "17");
    fourslash::verify_current_line_content(&mut s, r#"        } while (a !== b)"#);
    fourslash::go_to_marker(&mut s, "18");
    fourslash::verify_current_line_content(&mut s, r#"    } while (a !== b)"#);
    fourslash::go_to_marker(&mut s, "19");
    fourslash::verify_current_line_content(&mut s, r#"} while (a !== b)"#);
    fourslash::go_to_marker(&mut s, "20");
    fourslash::verify_current_line_content(&mut s, r#"for (var i = 0; i < 10; i++) {"#);
    fourslash::go_to_marker(&mut s, "21");
    fourslash::verify_current_line_content(&mut s, r#"    for (var j = 0; j < 10; j++) {"#);
    fourslash::go_to_marker(&mut s, "22");
    fourslash::verify_current_line_content(&mut s, r#"        j -= i"#);
    fourslash::go_to_marker(&mut s, "23");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "24");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "25");
    fourslash::verify_current_line_content(&mut s, r#"function foo() {"#);
    fourslash::go_to_marker(&mut s, "26");
    fourslash::verify_current_line_content(&mut s, r#"    try {"#);
    fourslash::go_to_marker(&mut s, "27");
    fourslash::verify_current_line_content(&mut s, r#"        x += 2"#);
    fourslash::go_to_marker(&mut s, "28");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "29");
    fourslash::verify_current_line_content(&mut s, r#"    catch (e) {"#);
    fourslash::go_to_marker(&mut s, "30");
    fourslash::verify_current_line_content(&mut s, r#"        x += 2"#);
    fourslash::go_to_marker(&mut s, "31");
    fourslash::verify_current_line_content(&mut s, r#"    } finally {"#);
    fourslash::go_to_marker(&mut s, "32");
    fourslash::verify_current_line_content(&mut s, r#"        x += 2"#);
    fourslash::go_to_marker(&mut s, "33");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "34");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "35");
    fourslash::verify_current_line_content(&mut s, r#"do { var a } while (1)"#);
    fourslash::go_to_marker(&mut s, "49");
    fourslash::verify_current_line_content(&mut s, r#"foo(function(file) {"#);
    fourslash::go_to_marker(&mut s, "50");
    fourslash::verify_current_line_content(&mut s, r#"    return 0"#);
    fourslash::go_to_marker(&mut s, "51");
    fourslash::verify_current_line_content(&mut s, r#"}).then(function(doc) {"#);
    fourslash::go_to_marker(&mut s, "52");
    fourslash::verify_current_line_content(&mut s, r#"    return 1"#);
    fourslash::go_to_marker(&mut s, "53");
    fourslash::verify_current_line_content(&mut s, r#"});"#);
    fourslash::go_to_marker(&mut s, "54");
    fourslash::verify_current_line_content(&mut s, r#"if (1)"#);
    fourslash::go_to_marker(&mut s, "55");
    fourslash::verify_current_line_content(&mut s, r#"    if (1)"#);
    fourslash::go_to_marker(&mut s, "56");
    fourslash::verify_current_line_content(&mut s, r#"        x++"#);
    fourslash::go_to_marker(&mut s, "57");
    fourslash::verify_current_line_content(&mut s, r#"    else"#);
    fourslash::go_to_marker(&mut s, "58");
    fourslash::verify_current_line_content(&mut s, r#"        if (1)"#);
    fourslash::go_to_marker(&mut s, "59");
    fourslash::verify_current_line_content(&mut s, r#"            x += 2"#);
    fourslash::go_to_marker(&mut s, "60");
    fourslash::verify_current_line_content(&mut s, r#"        else"#);
    fourslash::go_to_marker(&mut s, "61");
    fourslash::verify_current_line_content(&mut s, r#"            x += 2"#);
    fourslash::go_to_marker(&mut s, "62");
    fourslash::verify_current_line_content(&mut s, r#"                ;"#);
    fourslash::go_to_marker(&mut s, "63");
    fourslash::verify_current_line_content(&mut s, r#"do do do do"#);
    fourslash::go_to_marker(&mut s, "64");
    fourslash::verify_current_line_content(&mut s, r#"    test;"#);
    fourslash::go_to_marker(&mut s, "65");
    fourslash::verify_current_line_content(&mut s, r#"while (0)"#);
    fourslash::go_to_marker(&mut s, "66");
    fourslash::verify_current_line_content(&mut s, r#"while (0)"#);
    fourslash::go_to_marker(&mut s, "67");
    fourslash::verify_current_line_content(&mut s, r#"while (0)"#);
    fourslash::go_to_marker(&mut s, "68");
    fourslash::verify_current_line_content(&mut s, r#"while (0)"#);
}
