use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_property_symbols_from_base_types_doesnt_crash() {
    let content = r#"// @Filename: file1.ts
class ClassA implements IInterface {
    private [|value|]: number;
}"#;
    let mut s = Session::new_for_test("getPropertySymbolsFromBaseTypesDoesntCrash", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
