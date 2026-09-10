use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlight_in_type_export() {
    let content = r#"// @Filename: /1.ts
type [|A|] = 1;
export { [|A|] as [|B|] };
// @Filename: /2.ts
type [|A|] = 1;
let [|A|]: [|A|] = 1;
export { [|A|] as [|B|] };
// @Filename: /3.ts
type [|A|] = 1;
let [|A|]: [|A|] = 1;
export type { [|A|] as [|B|] };"#;
    let mut s = Session::new_for_test("documentHighlightInTypeExport", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
