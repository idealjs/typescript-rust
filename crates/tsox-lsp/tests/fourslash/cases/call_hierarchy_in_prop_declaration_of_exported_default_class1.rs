use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_in_prop_declaration_of_exported_default_class1() {
    let content = r#"// @Filename: /main.ts
export default class {
  onSave = () => {
    const values = [];
    values./*m1*/push(1);
  };
}
"#;
    let mut s = Session::new_for_test("callHierarchyInPropDeclarationOfExportedDefaultClass1", content);
    fourslash::go_to_marker(&mut s, "m1");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
