use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_property_description12() {
    let content = r#"type SymbolAlias = {
    /** Something generic */
    [p: symbol]: string;
}
function symbolAlias(e: SymbolAlias) {
    console.log(e./*symbolAlias*/anything);
}"#;
    let mut s = Session::new_for_test("jsDocPropertyDescription12", content);
    fourslash::verify_quick_info_at(&mut s, "symbolAlias", "any", "");
}
