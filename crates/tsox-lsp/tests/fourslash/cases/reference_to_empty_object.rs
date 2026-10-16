use tsox_lsp::fourslash::Session;


#[test]
fn reference_to_empty_object() {
    let content = r#"// @lib: es5
const obj = {}/*1*/;"#;
    let _s = Session::new_for_test("referenceToEmptyObject", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
