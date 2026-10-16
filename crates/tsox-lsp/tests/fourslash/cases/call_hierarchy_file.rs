use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_file() {
    let content = r#"foo();
function /**/foo() {
}"#;
    let mut s = Session::new_for_test("callHierarchyFile", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
