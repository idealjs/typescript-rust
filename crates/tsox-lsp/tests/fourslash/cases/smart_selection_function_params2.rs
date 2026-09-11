use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_function_params2() {
    let content = r#"function f(
  a,
  /**/b
) {}"#;
    let mut s = Session::new_for_test("smartSelection_functionParams2", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
