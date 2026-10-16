use tsox_lsp::fourslash::{self, Session};


#[test]
fn public_break() {
    let content = r#"public break;
/**/"#;
    let mut s = Session::new_for_test("publicBreak", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, " ");
}
