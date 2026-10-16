use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_anonymous_function_no_crash2() {
    let content = r#"// @Filename: /main.ts
(func/*1*/tion() {})"#;
    let mut s = Session::new_for_test("callHierarchyAnonymousFunctionNoCrash2", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
