use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn js_doc_property_description7() {
    let content = r#"class StringClass {
    /** Something generic */
    static [p: string]: any;
}
function stringClass(e: typeof StringClass) {
    console.log(e./*stringClass*/anything);
}"#;
    let mut s = Session::new_for_test("jsDocPropertyDescription7", content);
    fourslash::verify_quick_info_at(&mut s, "stringClass", "(index) StringClass[string]: any", "Something generic");
}
