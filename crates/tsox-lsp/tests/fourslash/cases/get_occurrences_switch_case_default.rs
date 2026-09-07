use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_switch_case_default() {
    let content = r#"[|switch|] (10) {
    [|case|] 1:
    [|case|] 2:
    [|case|] 4:
    [|case|] 8:
        foo: switch (20) {
            case 1:
            case 2:
                break;
            default:
                break foo;
        }
    [|case|] 0xBEEF:
    [|default|]:
        [|break|];
    [|case|] 16:
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
