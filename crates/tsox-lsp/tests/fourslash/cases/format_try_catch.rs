use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_try_catch() {
    let content = r#"function test() {
    /*try*/try {
    }
    /*catch*/catch (e) {
    }
}"#;
    let mut s = Session::new_for_test("formatTryCatch", content);
    fourslash::format_document(&mut s, "");
    fourslash::format_document(&mut s, "");
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "try");
    fourslash::verify_current_line_content(&mut s, r#"    try {"#);
    fourslash::go_to_marker(&mut s, "catch");
    fourslash::verify_current_line_content(&mut s, r#"    catch (e) {"#);
    // TODO: }
}
