use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.InsertLine"]
#[test]
fn format_on_enter_in_comment() {
    let content = r#"   /**
    * /*1*/
    */"#;
    let mut s = Session::new_for_test("formatOnEnterInComment", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::verify_current_file_content(&mut s, r#"  /**
   * 

   */"#);
}
