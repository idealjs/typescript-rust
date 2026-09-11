use tsox_lsp::fourslash::{self, Session};


#[test]
fn occurrences01() {
    let content = r#"// @lib: es5
foo: [|switch|] (10) {
    [|case|] 1:
    [|case|] 2:
    [|case|] 3:
        [|break|];
        [|break|] foo;
        continue;
        continue foo;
}"#;
    let mut s = Session::new_for_test("occurrences01", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
