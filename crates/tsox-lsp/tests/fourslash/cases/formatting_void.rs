use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_void() {
    let content = r#"/*1*/  var x: () =>           void    ;
/*2*/  var y:     void    ;
/*3*/  function test(a:void,b:string){}
/*4*/  var a, b, c, d;
/*5*/  void    a    ;
/*6*/  void        (0);
/*7*/  b=void(c=1,d=2);"#;
    let mut s = Session::new_for_test("formattingVoid", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"var x: () => void;"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"var y: void;"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"function test(a: void, b: string) { }"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"void a;"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"void (0);"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"b = void (c = 1, d = 2);"#);
}
