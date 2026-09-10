use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
#[test]
fn smart_selection_simple1() {
    let content = r#"class Foo {
  bar(a, b) {
      if (/*1*/a === b) {
          return tr/*2*/ue;
      }
      return false;
  }
}"#;
    let mut s = Session::new_for_test("smartSelection_simple1", content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
