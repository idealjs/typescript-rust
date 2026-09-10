use tsox_lsp::fourslash::{self, Session};


#[ignore = "needs live LSP session"]
#[test]
fn white_space_trimming4() {
    let content = r#"var re = /\w+   /*1*//;"#;
    let mut s = Session::new_for_test("whiteSpaceTrimming4", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "\n");
    fourslash::verify_current_file_content(&mut s, "var re = /\\w+\n    /;");
}
