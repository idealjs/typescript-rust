use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_incoming_calls_object_literal_method_in_identifier_computed_property() {
    let content = r#"const key = "x";
const obj = {
  [key]: {
    method() {
      return ""./*split*/split(",");
    }
  }
};
"#;
    let mut s = Session::new_for_test("callHierarchyIncomingCallsObjectLiteralMethodInIdentifierComputedProperty", content);
    fourslash::go_to_marker(&mut s, "split");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
