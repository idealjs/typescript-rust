use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_property_description5() {
    let content = r#"interface Multiple1Example {
    /** Something generic */
    [key: number | symbol | `data-${string}` | `data-${number}`]: string;
}
function multiple1Example(e: Multiple1Example) {
    console.log(e./*multiple1*/anything);
}"#;
    let mut s = Session::new_for_test("jsDocPropertyDescription5", content);
    fourslash::verify_quick_info_at(&mut s, "multiple1", "any", "");
}
