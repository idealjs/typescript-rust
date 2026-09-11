use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlight_at_inherited_properties5() {
    let content = r#"// @Filename: file1.ts
interface C extends D {
    [|prop0|]: string;
    [|prop1|]: number;
}

interface D extends C {
    [|prop0|]: string;
    [|prop1|]: number;
}

var d: D;
d.[|prop1|];"#;
    let mut s = Session::new_for_test("documentHighlightAtInheritedProperties5", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
