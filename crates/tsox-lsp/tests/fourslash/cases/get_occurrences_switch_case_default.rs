use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("getOccurrencesSwitchCaseDefault", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
