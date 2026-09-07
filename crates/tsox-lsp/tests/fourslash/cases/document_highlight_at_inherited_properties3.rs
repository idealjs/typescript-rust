use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
