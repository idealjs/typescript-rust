use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_equals_before_bracket_in_type_alias() {
    let content = r#"type X    =     [number]/*1*/"#;
    let mut s = Session::new_for_test("formattingEqualsBeforeBracketInTypeAlias", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"type X = [number];"#);
}
