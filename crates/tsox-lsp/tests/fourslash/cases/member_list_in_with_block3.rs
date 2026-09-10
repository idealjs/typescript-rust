use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_in_with_block3() {
    let content = r#"var x = { a: 0 };
with(x./*1*/"#;
    let mut s = Session::new_for_test("memberListInWithBlock3", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["a"]);
}
