use tsox_lsp::fourslash::{self, Session};


#[test]
fn folding_range_jsx_property_access() {
    let content = r#"// @jsx: preserve
// @Filename: /a.tsx
const Components =[| {
  Nested: () => null
}|];

export const Test = () =>[| {
  return [|<Components.Nested></Components.Nested>|];
}|];"#;
    let mut s = Session::new_for_test("foldingRangeJSXPropertyAccess", content);
    // TODO: f.VerifyOutliningSpans(t)
}
