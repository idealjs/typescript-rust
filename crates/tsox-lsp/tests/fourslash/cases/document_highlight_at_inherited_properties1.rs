use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_at_inherited_properties1() {
    let content = r#"// @Filename: file1.ts
interface interface1 extends interface1 {
   [|doStuff|](): void;
   [|propName|]: string;
}"#;
    let _s = Session::new_for_test("documentHighlightAtInheritedProperties1", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
