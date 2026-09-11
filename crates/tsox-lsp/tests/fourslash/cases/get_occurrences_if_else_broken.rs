use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_if_else_broken() {
    let content = r#"[|if|] (true) {
    var x = 1;
}
[|else     if|] ()
[|else if|]
[|else|]  /*  whar garbl   */   [|if|] (i/**/f (true) { } else { })
else"#;
    let mut s = Session::new_for_test("getOccurrencesIfElseBroken", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
