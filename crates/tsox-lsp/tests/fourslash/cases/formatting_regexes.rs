use tsox_lsp::fourslash::{self, Session};

#[test]
fn formatting_regexes() {
    let content = r#"removeAllButLast(sortedTypes, undefinedType, /keepNullableType**/ true)/*1*/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(
        &mut s,
        r#"removeAllButLast(sortedTypes, undefinedType, /keepNullableType**/ true);"#,
    );
}
