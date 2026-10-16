use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_if_else3() {
    let content = r#"if (true) {
    if (false) {
    }
    else {
    }
    [|if|] (true) {
    }
    [|else|] {
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
    let _s = Session::new_for_test("getOccurrencesIfElse3", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
