use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_on_function_property_returned_from_generic_function1() {
    let content = r#"function createProps<T>(t: T) {
  function getProps() {}
  function createVariants() {}

  getProps.createVariants = createVariants;
  return getProps;
}

createProps({})./**/createVariants();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(property) getProps<{}>.createVariants: () => void", "")
}
