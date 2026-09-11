use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_js_doc() {
    let content = r#"// Not a JSDoc comment
/**
 * @param {number} x The number to square
 */
function /**/square(x) {
  return x * x;
}"#;
    let mut s = Session::new_for_test("smartSelection_JSDoc", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
