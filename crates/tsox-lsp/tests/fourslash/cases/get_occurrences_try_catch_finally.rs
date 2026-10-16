use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_try_catch_finally() {
    let content = r#"/*1*/[|try|] {
    try {
    }
    catch (x) {
    }

    try {
    }
    finally {
    }
}
[|cat/*2*/ch|] (e) {
}
[|fina/*3*/lly|] {
}"#;
    let _s = Session::new_for_test("getOccurrencesTryCatchFinally", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
