use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_class_property_arrow_function() {
    let content = r#"class C {
    caller = () => {
        this.callee();
    }

    /**/callee = () => {
    }
}"#;
    let mut s = Session::new_for_test("callHierarchyClassPropertyArrowFunction", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
