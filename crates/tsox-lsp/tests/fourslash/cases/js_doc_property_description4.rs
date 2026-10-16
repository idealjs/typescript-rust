use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn js_doc_property_description4() {
    let content = r#"interface MultipleExample {
    /** Something generic */
    [key: string | number | symbol]: string;
}
function multipleExample(e: MultipleExample) {
    console.log(e./*multiple*/anything);
}"#;
    let mut s = Session::new_for_test("jsDocPropertyDescription4", content);
    fourslash::verify_quick_info_at(&mut s, "multiple", "(index) MultipleExample[string | number | symbol]: string", "Something generic");
}
