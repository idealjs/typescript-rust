use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn js_doc_property_description2() {
    let content = r#"interface SymbolExample {
    /** Something generic */
    [key: symbol]: string;
}
function symbolExample(e: SymbolExample) {
    console.log(e./*symbol*/anything);
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "symbol", "any", "")
}
