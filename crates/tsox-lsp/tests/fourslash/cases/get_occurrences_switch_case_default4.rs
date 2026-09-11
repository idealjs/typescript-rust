use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_switch_case_default4() {
    let content = r#"foo: [|switch|] (10) {
    [|case|] 1:
    [|case|] 2:
    [|case|] 3:
        [|break|];
        [|break|] foo;
        co/*1*/ntinue;
        contin/*2*/ue foo;
}"#;
    let mut s = Session::new_for_test("getOccurrencesSwitchCaseDefault4", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
