use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_try_catch_finally2() {
    let content = r#"try {
    [|t/*1*/r/*2*/y|] {
    }
    [|c/*3*/atch|] (x) {
    }

    try {
    }
    finally {
    }
}
catch (e) {
}
finally {
}"#;
    let _s = Session::new_for_test("getOccurrencesTryCatchFinally2", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
