use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_fat_arrow() {
    let content = r#"var items = [0, 1, 2];
items.forEach((n) => {
    /**/
    var q = n;
});"#;
    let mut s = Session::new_for_test("completionListInFatArrow", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "it");
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["items"], &[]);
}
