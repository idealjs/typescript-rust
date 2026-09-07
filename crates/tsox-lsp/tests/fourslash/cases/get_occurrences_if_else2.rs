use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_if_else2() {
    let content = r#"if (true) {
    [|if|] (false) {
    }
    [|else|]{
    }
    if (true) {
    }
    else {
        if (false)
            if (true)
                var x = undefined;
    }
}
else            if (null) {
}
else /* whar garbl */ if (undefined) {
}
else
if (false) {
}
else { }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
