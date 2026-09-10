use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
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
    let mut s = Session::new_for_test("getOccurrencesTryCatchFinally2", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
