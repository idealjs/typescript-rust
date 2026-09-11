use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_at_eof() {
    let content = r#"var a;"#;
    let mut s = Session::new_for_test("completionListAtEOF", content);
    // TODO: f.GoToEOF(t)
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["a"], &[]);
    // TODO: f.InsertLine(t, "")
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["a"], &[]);
    // TODO: f.InsertLine(t, "")
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["a"], &[]);
}
