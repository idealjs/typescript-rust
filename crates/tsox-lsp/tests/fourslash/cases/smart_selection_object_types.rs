use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_object_types() {
    let content = r#"type X = {
  /*1*/foo?: string;
  /*2*/readonly /*3*/bar: { x: num/*4*/ber };
  /*5*/meh
}"#;
    let _s = Session::new_for_test("smartSelection_objectTypes", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
