use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_exports() {
    let content = r#"export { a } from "a";

export { b as B } from "a" 

export import e = require("a");

export * from "a"; // no bindings here"#;
    let mut s = Session::new_for_test("navigationBarItemsExports", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
