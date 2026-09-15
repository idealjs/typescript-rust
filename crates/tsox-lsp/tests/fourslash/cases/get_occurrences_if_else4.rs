use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_if_else4() {
    let content = r#"if (true) {
    if (false) {
    }
    else {
    }
    if (true) {
    }
    else {
        /*1*/if (false)
            /*2*/i/*3*/f (true)
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
    let _s = Session::new_for_test("getOccurrencesIfElse4", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
