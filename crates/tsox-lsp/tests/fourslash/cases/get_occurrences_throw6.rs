use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_throw6() {
    let content = r#"[|throw|] 100;

try {
    throw 0;
    var x = () => { throw 0; };
}
catch (y) {
    var x = () => { throw 0; };
    [|throw|] 200;
}
finally {
    [|throw|] 300;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
