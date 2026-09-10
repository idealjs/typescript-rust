use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineCallHierarchy"]
#[test]
fn call_hierarchy_file() {
    let content = r#"foo();
function /**/foo() {
}"#;
    let mut s = Session::new_for_test("callHierarchyFile", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyBaselineCallHierarchy"); // f.VerifyBaselineCallHierarchy(t)
}
