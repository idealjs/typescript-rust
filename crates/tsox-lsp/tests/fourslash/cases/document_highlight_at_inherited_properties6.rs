use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlight_at_inherited_properties6() {
    let content = r#"// @Filename: file1.ts
class C extends D {
    [|prop0|]: string;
    [|prop1|]: string;
}

class D extends C {
    [|prop0|]: string;
    [|prop1|]: string;
}

var d: D;
d.[|prop1|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
