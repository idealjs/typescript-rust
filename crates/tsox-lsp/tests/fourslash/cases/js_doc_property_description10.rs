use tsox_lsp::fourslash::{self, Session};

#[test]
fn js_doc_property_description10() {
    let content = r#"class MultipleClass {
    /** Something generic */
    [key: number | symbol | ` + "`" + `data-${string}` + "`" + ` | ` + "`" + `data-${number}` + "`" + `]: string;
}
function multipleClass(e: typeof MultipleClass) {
    console.log(e./*multipleClass*/anything);
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "multipleClass", "any", "");
}
