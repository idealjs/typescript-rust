use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(
        &mut s,
        "",
        "(property) getProps<{}>.createVariants: () => void",
        "",
    );
}
