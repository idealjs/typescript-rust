use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_empty_ranges() {
    let content = r#"class HomePage {
  componentDidMount(/*1*/) {
    if (this.props.username/*2*/) {
      return '/*3*/';
    }
  }
}"#;
    let _s = Session::new_for_test("smartSelection_emptyRanges", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
