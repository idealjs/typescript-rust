use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_items2() {
    let content = r#"/**/"#;
    let mut s = Session::new_for_test("navigationBarItemsItems2", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert_line(&mut s, "module A");
    fourslash::insert(&mut s, "export class ");
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
