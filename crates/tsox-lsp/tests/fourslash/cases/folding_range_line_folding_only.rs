use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // Line 0: if (EMPTY_TAGs.has(tag)) {"]
#[test]
fn folding_range_line_folding_only() {
    let content = r#"if (EMPTY_TAGs.has(tag)) {
  output += "/>";
} else {
  output += ">";

  if (!html && kidcount > 0) {
    //
  }
}

export function use<T>(ctx: any): T | undefined {
  //
}"#;
    // TODO: ptrTrue := true
    // TODO: capabilities := &lsproto.ClientCapabilities{
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: // With lineFoldingOnly, end lines should be adjusted so closing brackets stay visible.
    // TODO: // Line 0: if (EMPTY_TAGs.has(tag)) {
    // TODO: // Line 9:
    // TODO: // Line 10: export function use<T>(ctx: any): T | undefined {
    fourslash::unsupported("VerifyFoldingRangeLines"); // f.VerifyFoldingRangeLines(t, []fourslash.FoldingRangeLineExpected{
}

#[ignore = "generator: // Line 0: // #region MyRegion"]
#[test]
fn folding_range_line_folding_only_with_regions() {
    let content = r#"// #region MyRegion
const x = 1;
function foo() {
  return x;
}
// #endregion

// #region Outer
const y = 2;
// #region Inner
const z = 3;
// #endregion
// #endregion"#;
    // TODO: ptrTrue := true
    // TODO: capabilities := &lsproto.ClientCapabilities{
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: // Line 0: // #region MyRegion
    // TODO: // Line 1: const x = 1;
    // TODO: // Line 2: function foo() {
    // TODO: // Line 5: // #endregion
    // TODO: // Line 6:
    // TODO: // Line 7: // #region Outer
    // TODO: // Line 8: const y = 2;
    // TODO: // Line 9: // #region Inner
    // TODO: // Line 10: const z = 3;
    // TODO: // Line 11: // #endregion
    // TODO: // Line 12: // #endregion
    fourslash::unsupported("VerifyFoldingRangeLines"); // f.VerifyFoldingRangeLines(t, []fourslash.FoldingRangeLineExpected{
}
