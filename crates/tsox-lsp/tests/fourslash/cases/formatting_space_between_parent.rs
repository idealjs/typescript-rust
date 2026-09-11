use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_space_between_parent() {
    let content = r#"/*1*/foo(() => 1);
/*2*/foo(1);
/*3*/if((true)){}"#;
    let mut s = Session::new_for_test("formattingSpaceBetweenParent", content);
    // TODO: opts180 := f.GetOptions()
    // TODO: opts180.FormatCodeSettings.InsertSpaceAfterOpeningAndBeforeClosingNonemptyParenthesis = core.TSTrue
    // TODO: f.Configure(t, opts180)
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"foo( () => 1 );"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"foo( 1 );"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"if ( ( true ) ) { }"#);
}
