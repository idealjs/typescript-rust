use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_inside_with_block() {
    let content = r#"/*1*/var /*2*/x = 0;

with ({}) {
    var y = x;  // Reference of x here should not be picked
    y++;        // also reference for y should be ignored
}

/*3*/x = /*4*/x + 1;"#;
    let mut s = Session::new_for_test("findAllRefsInsideWithBlock", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
