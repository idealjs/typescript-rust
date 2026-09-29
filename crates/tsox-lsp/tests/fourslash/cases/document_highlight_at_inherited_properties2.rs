use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_at_inherited_properties2() {
    let content = r#"// @Filename: file1.ts
class class1 extends class1 {
   [|doStuff|]() { }
   [|propName|]: string;
}"#;
    let _s = Session::new_for_test("documentHighlightAtInheritedProperties2", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
