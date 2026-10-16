use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_incoming_calls_object_literal_method_in_expression_computed_property() {
    let content = r#"const obj = {
  [1 + 2]: {
    method() {
      return ""./*split*/split(",");
    }
  }
};
"#;
    let mut s = Session::new_for_test("callHierarchyIncomingCallsObjectLiteralMethodInExpressionComputedProperty", content);
    fourslash::go_to_marker(&mut s, "split");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
