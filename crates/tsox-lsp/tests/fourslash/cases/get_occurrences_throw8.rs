use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_throw8() {
    let content = r#"try {
    throw 10;

    try {
        [|throw|] 10;
    }
    catch (x) {
        throw 10;
    }
    finally {
        throw 10;
    }
}
finally {
    throw 10;
}

throw 10;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
