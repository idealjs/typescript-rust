use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn js_doc_property_description8() {
    let content = r#"class SymbolClass {
    /** Something generic */
    static [p: symbol]: any;
}
function symbolClass(e: typeof SymbolClass) {
    console.log(e./*symbolClass*/anything);
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "symbolClass", "any", "")
}
