use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_property_description3() {
    let content = r#"interface LiteralExample {
    /** Something generic */
    [key: `data-${string}`]: string;
     /** Something else */
    [key: `prefix${number}`]: number;
}
function literalExample(e: LiteralExample) {
    console.log(e./*literal*/anything);
}"#;
    let mut s = Session::new_for_test("jsDocPropertyDescription3", content);
    fourslash::verify_quick_info_at(&mut s, "literal", "any", "");
}
