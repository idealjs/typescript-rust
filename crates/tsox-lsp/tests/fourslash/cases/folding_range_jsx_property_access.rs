use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
