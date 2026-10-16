use tsox_lsp::fourslash::{self, Session};


#[test]
fn memberlist_on_d_dot() {
    let content = r#"var q = '';
q/**/"#;
    let mut s = Session::new_for_test("memberlistOnDDot", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ".");
    fourslash::insert(&mut s, ".");
    fourslash::verify_completions_empty_at(&mut s, None);
}
