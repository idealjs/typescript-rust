use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn js_doc_property_description11() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"type AliasExample = {
    /** Something generic */
    [p: string]: string;
    /** Something else */
    [key: ` + "`" + `any${string}` + "`" + `]: string;
}
function aliasExample(e: AliasExample) {
    console.log(e./*alias*/anything);
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "alias", "(index) AliasExample[string | `any${string}`]: string", "Something 
}
