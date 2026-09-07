use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
#[test]
fn smart_selection_empty_ranges() {
    let content = r#"class HomePage {
  componentDidMount(/*1*/) {
    if (this.props.username/*2*/) {
      return '/*3*/';
    }
  }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
