use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_at_inherited_properties3() {
    let content = r#"// @Filename: file1.ts
interface interface1 extends interface1 {
   [|doStuff|](): void;
   [|propName|]: string;
}

var v: interface1;
v.[|propName|];
v.[|doStuff|]();"#;
    let _s = Session::new_for_test("documentHighlightAtInheritedProperties3", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
