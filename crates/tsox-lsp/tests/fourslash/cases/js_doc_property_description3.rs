use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn js_doc_property_description3() {
    let content = r#"interface LiteralExample {
    /** Something generic */
    [key: ` + "`" + `data-${string}` + "`" + `]: string;
     /** Something else */
    [key: ` + "`" + `prefix${number}` + "`" + `]: number;
}
function literalExample(e: LiteralExample) {
    console.log(e./*literal*/anything);
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "literal", "any", "")
}
