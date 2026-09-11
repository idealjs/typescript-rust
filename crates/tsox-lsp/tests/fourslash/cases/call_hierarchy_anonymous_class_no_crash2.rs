use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_anonymous_class_no_crash2() {
    let content = r#"// @Filename: /main.ts
(class {
    con/*1*/structor() {}
})"#;
    let mut s = Session::new_for_test("callHierarchyAnonymousClassNoCrash2", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
