use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlight_at_inherited_properties4() {
    let content = r#"// @Filename: file1.ts
class class1 extends class1 {
   [|doStuff|]() { }
   [|propName|]: string;
}

var c: class1;
c.[|doStuff|]();
c.[|propName|];"#;
    let mut s = Session::new_for_test("documentHighlightAtInheritedProperties4", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
