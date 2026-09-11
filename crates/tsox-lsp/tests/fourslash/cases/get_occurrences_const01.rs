use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_const01() {
    let content = r#"[|const|] enum E1 {
    v1,
    v2
}

/*2*/const c = 0;"#;
    let mut s = Session::new_for_test("getOccurrencesConst01", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "2")
}
