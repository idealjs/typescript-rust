use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_property_description8() {
    let content = r#"class SymbolClass {
    /** Something generic */
    static [p: symbol]: any;
}
function symbolClass(e: typeof SymbolClass) {
    console.log(e./*symbolClass*/anything);
}"#;
    let mut s = Session::new_for_test("jsDocPropertyDescription8", content);
    fourslash::verify_quick_info_at(&mut s, "symbolClass", "any", "");
}
