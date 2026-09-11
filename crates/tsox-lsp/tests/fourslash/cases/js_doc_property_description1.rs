use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_property_description1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface StringExample {
    /** Something generic */
    [p: string]: any; 
    /** Something specific */
    property: number;
}
function stringExample(e: StringExample) {
    console.log(e./*property*/property);
    console.log(e./*string*/anything); 
}"#;
    let mut s = Session::new_for_test("jsDocPropertyDescription1", content);
    fourslash::verify_quick_info_at(&mut s, "property", "(property) StringExample.property: number", "Something specific");
    fourslash::verify_quick_info_at(&mut s, "string", "(index) StringExample[string]: any", "Something generic");
}
