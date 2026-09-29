use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_interface_method() {
    let content = r#"interface I {
    /**/foo(): void;
}

const obj: I = { foo() {} };

obj.foo();"#;
    let mut s = Session::new_for_test("callHierarchyInterfaceMethod", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
