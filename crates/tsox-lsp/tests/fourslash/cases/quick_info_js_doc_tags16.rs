use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_js_doc_tags16() {
    let content = r#"class A {
    /**
     * Description text here.
     *
     * @virtual
     */
    foo() { }
}

class B extends A {
    override /*1*/foo() { }
}

class C extends B {
    override /*2*/foo() { }
}"#;
    let mut s = Session::new_for_test("quickInfoJsDocTags16", content);
    // TODO: f.VerifyBaselineHover(t)
}
