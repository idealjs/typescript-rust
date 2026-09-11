use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_try_catch_finally3() {
    let content = r#"try {
    try {
    }
    catch (x) {
    }

    [|t/*1*/r/*2*/y|] {
    }
    [|finall/*3*/y|] {
    }
}
catch (e) {
}
finally {
}"#;
    let mut s = Session::new_for_test("getOccurrencesTryCatchFinally3", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
