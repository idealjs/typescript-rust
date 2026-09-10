use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_no_error_truncation1() {
    let content = r#"// @noErrorTruncation: true
type /*1*/T = [
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
  'still good', 'now truncating'
];"#;
    let mut s = Session::new_for_test("quickinfoVerbosityNoErrorTruncation1", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
