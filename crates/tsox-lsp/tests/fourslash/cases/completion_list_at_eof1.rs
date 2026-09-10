use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.GoToEOF"]
#[test]
fn completion_list_at_eof1() {
    let content = r#"if(0 === ''."#;
    let mut s = Session::new_for_test("completionListAtEOF1", content);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["charAt"], &[]);
}
