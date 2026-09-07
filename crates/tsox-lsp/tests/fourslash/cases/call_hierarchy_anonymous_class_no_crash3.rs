use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineCallHierarchy"]
#[test]
fn call_hierarchy_anonymous_class_no_crash3() {
    let content = r#"// @Filename: /main.ts
import Bar from "./other";

function foo() {
    new /*1*/Bar();
}
// @Filename: /other.ts
export default class {
    constructor() {}
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyBaselineCallHierarchy"); // f.VerifyBaselineCallHierarchy(t)
}
