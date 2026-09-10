use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
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
    let mut s = Session::new_for_test("getOccurrencesTryCatchFinally", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
