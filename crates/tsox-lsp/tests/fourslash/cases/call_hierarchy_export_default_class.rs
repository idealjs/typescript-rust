use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_export_default_class() {
    let content = r#"// @filename: main.ts
import Bar from "./other";

function foo() {
    new Bar();
}
// @filename: other.ts
export /**/default class {
    constructor() {
        baz();
    }
}

function baz() {
}"#;
    let mut s = Session::new_for_test("callHierarchyExportDefaultClass", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
