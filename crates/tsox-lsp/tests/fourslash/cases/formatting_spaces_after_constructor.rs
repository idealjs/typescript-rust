use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_spaces_after_constructor() {
    let content = r#"/*1*/class test { constructor                   () { } }
/*2*/class test { constructor                   () { } }"#;
    let mut s = Session::new_for_test("formattingSpacesAfterConstructor", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"class test { constructor() { } }"#);
    // TODO: opts319 := f.GetOptions()
    fourslash::configure_format_settings(&mut s, &[("insert_space_after_constructor", "true")]);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"class test { constructor () { } }"#);
}
