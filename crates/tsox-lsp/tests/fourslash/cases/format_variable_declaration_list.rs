use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_variable_declaration_list() {
    let content = r#"/*1*/var   fun1   =   function   (     )     {
/*2*/            var               x   =   'foo'             ,
/*3*/                z   =   'bar'           ;
/*4*/                return  x            ;
/*5*/},

/*6*/fun2   =   (                function        (   f               )   {
/*7*/            var   fun   =   function   (        )       {
/*8*/                        console         .  log             (           f     (  )  )       ;
/*9*/            },
/*10*/            x   =   'Foo'           ;
/*11*/                return   fun            ;
/*12*/}   (           fun1            )   )       ;"#;
    let mut s = Session::new_for_test("formatVariableDeclarationList", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"var fun1 = function() {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    var x = 'foo',"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"        z = 'bar';"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"    return x;"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"},"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"    fun2 = (function(f) {"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"        var fun = function() {"#);
    fourslash::go_to_marker(&mut s, "8");
    fourslash::verify_current_line_content(&mut s, r#"            console.log(f());"#);
    fourslash::go_to_marker(&mut s, "9");
    fourslash::verify_current_line_content(&mut s, r#"        },"#);
    fourslash::go_to_marker(&mut s, "10");
    fourslash::verify_current_line_content(&mut s, r#"            x = 'Foo';"#);
    fourslash::go_to_marker(&mut s, "11");
    fourslash::verify_current_line_content(&mut s, r#"        return fun;"#);
    fourslash::go_to_marker(&mut s, "12");
    fourslash::verify_current_line_content(&mut s, r#"    }(fun1));"#);
}
