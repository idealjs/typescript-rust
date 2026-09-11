use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_const03() {
    let content = r#"namespace m {
    export /*1*/const x;
    export [|const|] enum E {
    }
}

export /*2*/const x;
export [|const|] enum E {
}"#;
    let mut s = Session::new_for_test("getOccurrencesConst03", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
