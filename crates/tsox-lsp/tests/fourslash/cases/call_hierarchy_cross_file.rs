use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_cross_file() {
    let content = r#"// @filename: /a.ts
export function /**/createModelReference() {}
// @filename: /b.ts
import { createModelReference } from "./a";
function openElementsAtEditor() {
  createModelReference();
}
// @filename: /c.ts
import { createModelReference } from "./a";
function registerDefaultLanguageCommand() {
  createModelReference();
}"#;
    let mut s = Session::new_for_test("callHierarchyCrossFile", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
