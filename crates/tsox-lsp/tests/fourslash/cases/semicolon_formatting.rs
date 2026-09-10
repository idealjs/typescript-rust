use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn semicolon_formatting() {
    let content = r#"/**/function of1 (b:{r:{c:number"#;
    let mut s = Session::new_for_test("semicolonFormatting", content);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"function of1(b: { r: { c: number;"#);
    // TODO: }
}
