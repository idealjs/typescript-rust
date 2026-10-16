use tsox_lsp::fourslash::{self, Session};


#[test]
fn semicolon_formatting() {
    let content = r#"/**/function of1 (b:{r:{c:number"#;
    let mut s = Session::new_for_test("semicolonFormatting", content);
    fourslash::go_to_eof(&mut s, );
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"function of1(b: { r: { c: number;"#);
    // TODO: }
}
