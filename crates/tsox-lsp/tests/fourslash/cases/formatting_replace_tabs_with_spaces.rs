use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_replace_tabs_with_spaces() {
    let content = r#"namespace Foo {
/*1*/				class Test { }
/*2*/			class Test { }
/*3*/class Test { }
/*4*/			 class Test { }
/*5*/   class Test { }
/*6*/    class Test { }
/*7*/     class Test { }
}"#;
    let mut s = Session::new_for_test("formattingReplaceTabsWithSpaces", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"    class Test { }"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    class Test { }"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"    class Test { }"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"    class Test { }"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"    class Test { }"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"    class Test { }"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"    class Test { }"#);
}
