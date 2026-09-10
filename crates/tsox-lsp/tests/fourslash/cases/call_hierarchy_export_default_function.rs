use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineCallHierarchy"]
#[test]
fn call_hierarchy_export_default_function() {
    let content = r#"// @filename: main.ts
import bar from "./other";

function foo() {
    bar();
}
// @filename: other.ts
export /**/default function () {
    baz();
}

function baz() {
}"#;
    let mut s = Session::new_for_test("callHierarchyExportDefaultFunction", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyBaselineCallHierarchy"); // f.VerifyBaselineCallHierarchy(t)
}
