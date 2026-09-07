use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineCallHierarchy"]
#[test]
fn call_hierarchy_anonymous_function_no_crash3() {
    let content = r#"// @Filename: /main.ts
import bar from "./other";

function foo() {
    /*1*/bar();
}
// @Filename: /other.ts
export default function() {}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyBaselineCallHierarchy"); // f.VerifyBaselineCallHierarchy(t)
}
