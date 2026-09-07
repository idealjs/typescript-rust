use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.InsertLine"]
#[test]
fn navigation_bar_items_items2() {
    let content = r#"/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "module A")
    fourslash::insert(&mut s, "export class ");
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
