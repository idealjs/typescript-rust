use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_incremental() {
    let content = r#"/**/"#;
    let mut s = Session::new_for_test("tsxIncremental", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "<");
    fourslash::insert(&mut s, "div");
    fourslash::insert(&mut s, " ");
    fourslash::insert(&mut s, " id");
    fourslash::insert(&mut s, "=");
    fourslash::insert(&mut s, "\"foo");
    fourslash::insert(&mut s, "\"");
    fourslash::insert(&mut s, ">");
}
