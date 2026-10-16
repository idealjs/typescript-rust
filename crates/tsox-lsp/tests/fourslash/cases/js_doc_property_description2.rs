use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_property_description2() {
    let content = r#"interface SymbolExample {
    /** Something generic */
    [key: symbol]: string;
}
function symbolExample(e: SymbolExample) {
    console.log(e./*symbol*/anything);
}"#;
    let mut s = Session::new_for_test("jsDocPropertyDescription2", content);
    fourslash::verify_quick_info_at(&mut s, "symbol", "any", "");
}
