use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_throw7() {
    let content = r#"try {
    [|throw|] 10;

    try {
        throw 10;
    }
    catch (x) {
        [|throw|] 10;
    }
    finally {
        [|throw|] 10;
    }
}
finally {
    [|throw|] 10;
}

[|throw|] 10;"#;
    let mut s = Session::new_for_test("getOccurrencesThrow7", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
