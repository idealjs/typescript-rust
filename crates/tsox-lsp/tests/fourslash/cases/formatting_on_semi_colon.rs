use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_semi_colon() {
    let content = r#"var  a=b+c^d-e*++f"#;
    let mut s = Session::new_for_test("formattingOnSemiColon", content);
    fourslash::go_to_eof(&mut s, );
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_file_content(&mut s, r#"var a = b + c ^ d - e * ++f;"#);
}
