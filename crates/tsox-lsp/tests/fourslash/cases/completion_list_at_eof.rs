use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_eof() {
    let content = r#"var a;"#;
    let mut s = Session::new_for_test("completionListAtEOF", content);
    fourslash::go_to_eof(&mut s, );
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["a"], &[]);
    fourslash::insert_line(&mut s, "");
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["a"], &[]);
    fourslash::insert_line(&mut s, "");
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["a"], &[]);
}
