use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_if_else() {
    let content = r#"[|if|] (true) {
    if (false) {
    }
    else {
    }
    if (true) {
    }
    else {
        if (false)
            if (true)
                var x = undefined;
    }
}
[|else            i/**/f|] (null) {
}
[|else|] /* whar garbl */ [|if|] (undefined) {
}
[|else|]
[|if|] (false) {
}
[|else|] { }"#;
    let _s = Session::new_for_test("getOccurrencesIfElse", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
