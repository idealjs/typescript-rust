use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_space_between_optional_chaining() {
    let content = r#"/*1*/a    ?.    b   ?.   c   .   d;
/*2*/o    .  m()   ?.   length;"#;
    let mut s = Session::new_for_test("formattingSpaceBetweenOptionalChaining", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"a?.b?.c.d;"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"o.m()?.length;"#);
}
