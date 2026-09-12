use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_in_unclosed_template02() {
    let content = r#"var x;
var y = (p) => `abc ${ 123 } ${ /*1*/"#;
    let mut s = Session::new_for_test("completionListInUnclosedTemplate02", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["p", "x"], &[]);
    // TODO: }
}
