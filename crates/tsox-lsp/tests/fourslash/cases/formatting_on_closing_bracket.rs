use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: opts874 := f.GetOptions()"]
#[test]
fn formatting_on_closing_bracket() {
    let content = r#"function f( ) {/*1*/
var     x = 3;/*2*/
    var z = 2   ;/*3*/
    a  = z  ++ - 2 *  x ;/*4*/
        for ( ; ; ) {/*5*/
    a+=(g +g)*a%t;/*6*/
        b --                          ;/*7*/
}/*8*/

    switch ( a  )/*9*/
    {
        case 1  :     {/*10*/
    a ++  ;/*11*/
        b--;/*12*/
    if(a===a)/*13*/
                return;/*14*/
    else/*15*/
        {
            for(a in b)/*16*/
                if(a!=a)/*17*/
    {
    for(a in b)/*18*/
            {
a++;/*19*/
        }/*20*/
                }/*21*/
    }/*22*/
        }/*23*/
    default:/*24*/
        break;/*25*/
    }/*26*/
}/*27*/"#;
    let mut s = Session::new_for_test("formattingOnClosingBracket", content);
    // TODO: opts874 := f.GetOptions()
    // TODO: opts874.FormatCodeSettings.InsertSpaceAfterSemicolonInForStatements = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts874)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"function f() {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    var x = 3;"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"    var z = 2;"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"    a = z++ - 2 * x;"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"    for (; ;) {"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"        a += (g + g) * a % t;"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"        b--;"#);
    fourslash::go_to_marker(&mut s, "8");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "9");
    fourslash::verify_current_line_content(&mut s, r#"    switch (a) {"#);
    fourslash::go_to_marker(&mut s, "10");
    fourslash::verify_current_line_content(&mut s, r#"        case 1: {"#);
    fourslash::go_to_marker(&mut s, "11");
    fourslash::verify_current_line_content(&mut s, r#"            a++;"#);
    fourslash::go_to_marker(&mut s, "12");
    fourslash::verify_current_line_content(&mut s, r#"            b--;"#);
    fourslash::go_to_marker(&mut s, "13");
    fourslash::verify_current_line_content(&mut s, r#"            if (a === a)"#);
    fourslash::go_to_marker(&mut s, "14");
    fourslash::verify_current_line_content(&mut s, r#"                return;"#);
    fourslash::go_to_marker(&mut s, "15");
    fourslash::verify_current_line_content(&mut s, r#"            else {"#);
    fourslash::go_to_marker(&mut s, "16");
    fourslash::verify_current_line_content(&mut s, r#"                for (a in b)"#);
    fourslash::go_to_marker(&mut s, "17");
    fourslash::verify_current_line_content(&mut s, r#"                    if (a != a) {"#);
    fourslash::go_to_marker(&mut s, "18");
    fourslash::verify_current_line_content(&mut s, r#"                        for (a in b) {"#);
    fourslash::go_to_marker(&mut s, "19");
    fourslash::verify_current_line_content(&mut s, r#"                            a++;"#);
    fourslash::go_to_marker(&mut s, "20");
    fourslash::verify_current_line_content(&mut s, r#"                        }"#);
    fourslash::go_to_marker(&mut s, "21");
    fourslash::verify_current_line_content(&mut s, r#"                    }"#);
    fourslash::go_to_marker(&mut s, "22");
    fourslash::verify_current_line_content(&mut s, r#"            }"#);
    fourslash::go_to_marker(&mut s, "23");
    fourslash::verify_current_line_content(&mut s, r#"        }"#);
    fourslash::go_to_marker(&mut s, "24");
    fourslash::verify_current_line_content(&mut s, r#"        default:"#);
    fourslash::go_to_marker(&mut s, "25");
    fourslash::verify_current_line_content(&mut s, r#"            break;"#);
    fourslash::go_to_marker(&mut s, "26");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "27");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
}
