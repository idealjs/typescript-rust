use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn js_doc_see3() {
    let content = r#"function foo ([|/*def1*/a|]: string) {
    /**
     * @see {/*use1*/[|a|]}
     */
    function bar ([|/*def2*/a|]: string) {
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "use1")
}
