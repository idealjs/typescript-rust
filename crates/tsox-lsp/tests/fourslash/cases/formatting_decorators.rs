use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn formatting_decorators() {
    let content = r#"/*1*/        @    decorator1    
/*2*/            @        decorator2
/*3*/    @decorator3
/*4*/        @    decorator4    @            decorator5
/*5*/class C {
/*6*/            @    decorator6    
/*7*/                @        decorator7
/*8*/        @decorator8
/*9*/    method1() { }

/*10*/        @    decorator9    @            decorator10 @decorator11            method2() { }

    method3(
/*11*/                @    decorator12    
/*12*/                    @        decorator13
/*13*/            @decorator14
/*14*/        x) { }

    method4(
/*15*/            @    decorator15    @            decorator16 @decorator17             x) { }

/*16*/            @    decorator18    
/*17*/                @        decorator19
/*18*/        @decorator20    
/*19*/    ["computed1"]() { }

/*20*/        @    decorator21    @            decorator22 @decorator23            ["computed2"]() { }

/*21*/            @    decorator24    
/*22*/                @        decorator25
/*23*/        @decorator26
/*24*/    get accessor1() { }

/*25*/        @    decorator27    @            decorator28 @decorator29            get accessor2() { }

/*26*/            @    decorator30    
/*27*/                @        decorator31
/*28*/        @decorator32
/*29*/    property1;

/*30*/        @    decorator33    @            decorator34 @decorator35            property2;
/*31*/function test(@decorator36@decorator37 param) {};
/*32*/function test2(@decorator38()@decorator39()param) {};
}"#;
    let mut s = Session::new_for_test("formattingDecorators", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"@decorator1"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"@decorator2"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"@decorator3"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"@decorator4 @decorator5"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"class C {"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator6"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator7"#);
    fourslash::go_to_marker(&mut s, "8");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator8"#);
    fourslash::go_to_marker(&mut s, "9");
    fourslash::verify_current_line_content(&mut s, r#"    method1() { }"#);
    fourslash::go_to_marker(&mut s, "10");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator9 @decorator10 @decorator11 method2() { }"#);
    fourslash::go_to_marker(&mut s, "11");
    fourslash::verify_current_line_content(&mut s, r#"        @decorator12"#);
    fourslash::go_to_marker(&mut s, "12");
    fourslash::verify_current_line_content(&mut s, r#"        @decorator13"#);
    fourslash::go_to_marker(&mut s, "13");
    fourslash::verify_current_line_content(&mut s, r#"        @decorator14"#);
    fourslash::go_to_marker(&mut s, "14");
    fourslash::verify_current_line_content(&mut s, r#"        x) { }"#);
    fourslash::go_to_marker(&mut s, "15");
    fourslash::verify_current_line_content(&mut s, r#"        @decorator15 @decorator16 @decorator17 x) { }"#);
    fourslash::go_to_marker(&mut s, "16");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator18"#);
    fourslash::go_to_marker(&mut s, "17");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator19"#);
    fourslash::go_to_marker(&mut s, "18");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator20"#);
    fourslash::go_to_marker(&mut s, "19");
    fourslash::verify_current_line_content(&mut s, r#"    ["computed1"]() { }"#);
    fourslash::go_to_marker(&mut s, "20");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator21 @decorator22 @decorator23 ["computed2"]() { }"#);
    fourslash::go_to_marker(&mut s, "21");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator24"#);
    fourslash::go_to_marker(&mut s, "22");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator25"#);
    fourslash::go_to_marker(&mut s, "23");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator26"#);
    fourslash::go_to_marker(&mut s, "24");
    fourslash::verify_current_line_content(&mut s, r#"    get accessor1() { }"#);
    fourslash::go_to_marker(&mut s, "25");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator27 @decorator28 @decorator29 get accessor2() { }"#);
    fourslash::go_to_marker(&mut s, "26");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator30"#);
    fourslash::go_to_marker(&mut s, "27");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator31"#);
    fourslash::go_to_marker(&mut s, "28");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator32"#);
    fourslash::go_to_marker(&mut s, "29");
    fourslash::verify_current_line_content(&mut s, r#"    property1;"#);
    fourslash::go_to_marker(&mut s, "30");
    fourslash::verify_current_line_content(&mut s, r#"    @decorator33 @decorator34 @decorator35 property2;"#);
    fourslash::go_to_marker(&mut s, "31");
    fourslash::verify_current_line_content(&mut s, r#"function test(@decorator36 @decorator37 param) { };"#);
    fourslash::go_to_marker(&mut s, "32");
    fourslash::verify_current_line_content(&mut s, r#"function test2(@decorator38() @decorator39() param) { };"#);
    // TODO: }
}
