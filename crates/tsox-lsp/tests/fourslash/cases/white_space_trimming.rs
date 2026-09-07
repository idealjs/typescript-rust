use tsox_lsp::fourslash::{self, Session};

#[ignore = "needs live LSP session"]
#[test]
fn white_space_trimming() {
    let content = r#"if (true) {     
  //    
   /*err*/}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "err");
    fourslash::insert(&mut s, "\n");
    fourslash::verify_current_file_content(
        &mut s,
        r#"if (true) {     
  //    

}"#,
    );
}
