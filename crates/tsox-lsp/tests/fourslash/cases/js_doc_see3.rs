use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_see3() {
    let content = r#"function foo ([|/*def1*/a|]: string) {
    /**
     * @see {/*use1*/[|a|]}
     */
    function bar ([|/*def2*/a|]: string) {
    }
}"#;
    let mut s = Session::new_for_test("jsDocSee3", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "use1")
}
