use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn js_doc_property_description7() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class StringClass {
    /** Something generic */
    static [p: string]: any;
}
function stringClass(e: typeof StringClass) {
    console.log(e./*stringClass*/anything);
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "stringClass", "(index) StringClass[string]: any", "Something generic")
}
