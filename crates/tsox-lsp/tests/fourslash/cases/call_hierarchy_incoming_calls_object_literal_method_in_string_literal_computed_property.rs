use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_incoming_calls_object_literal_method_in_string_literal_computed_property() {
    let content = r#"const obj = {
  ["x"]: {
    method() {
      return ""./*split*/split(",");
    }
  }
};
"#;
    let mut s = Session::new_for_test("callHierarchyIncomingCallsObjectLiteralMethodInStringLiteralComputedProperty", content);
    fourslash::go_to_marker(&mut s, "split");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
