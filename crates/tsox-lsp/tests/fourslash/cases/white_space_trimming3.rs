use tsox_lsp::fourslash::{self, Session};


#[test]
fn white_space_trimming3() {
    let content = r#"let t = "foo \
bar     \   
"/*1*/"#;
    let mut s = Session::new_for_test("whiteSpaceTrimming3", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_file_content(&mut s, "let t = \"foo \\\nbar     \\   \n\";");
}
