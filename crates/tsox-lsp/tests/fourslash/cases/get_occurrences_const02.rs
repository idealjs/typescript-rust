use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_const02() {
    let content = r#"namespace m {
    declare /*1*/const x;
    declare [|const|] enum E {
    }
}

declare /*2*/const x;
declare [|const|] enum E {
}"#;
    let _s = Session::new_for_test("getOccurrencesConst02", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
