use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_switch_case_default3() {
    let content = r#"foo: [|switch|] (1) {
    [|case|] 1:
    [|case|] 2:
        [|break|];
    [|case|] 3:
        switch (2) {
            case 1:
                [|break|] foo;
                continue; // invalid
            default:
                break;
        }
    [|default|]:
        [|break|];
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
