use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.GoToEOF"]
#[test]
fn completion_list_at_eof() {
    let content = r#"var a;"#;
    let mut s = Session::new_for_test("completionListAtEOF", content);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["a"], &[]);
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["a"], &[]);
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["a"], &[]);
}
