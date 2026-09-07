use tsox_lsp::fourslash::{self, Session};

#[test]
fn insert_second_try_catch_block() {
    let content = r#"try {} catch(e) { }
/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "try {} catch(e) { }");
}
