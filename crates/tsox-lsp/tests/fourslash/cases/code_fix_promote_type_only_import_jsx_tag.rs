use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_promote_type_only_import_jsx_tag() {
    let content = r#"// @module: preserve
// @verbatimModuleSyntax: true
// @jsx: react
// @Filename: /react.ts
const React: any = {};
export default React;
// @Filename: /bar.tsx
import type React from "./react";

<Foo/**/ />;"#;
    let mut s = Session::new_for_test("codeFixPromoteTypeOnlyImportJsxTag", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: // The fix should promote the type-only import of React to a regular import.
    // TODO: // The "Cannot find name 'Foo'" error does not produce an auto-import for
    // TODO: // React since it's already imported (as type-only, handled by promotion).
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}

#[test]
fn code_fix_promote_type_only_import_jsx_tag_both_type_only() {
    let content = r#"// @module: preserve
// @verbatimModuleSyntax: true
// @jsx: react
// @Filename: /react.ts
const React: any = {};
export default React;
// @Filename: /foo.ts
export function Foo() { return null; }
// @Filename: /bar.tsx
import type React from "./react";
import type { Foo } from "./foo";

<Foo/**/ />;"#;
    let mut s = Session::new_for_test("codeFixPromoteTypeOnlyImportJsxTagBothTypeOnly", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: // Both Foo and React are type-only imported. The error message string
    // TODO: // matching disambiguates which diagnostic is about which symbol, so each
    // TODO: // diagnostic produces only its own promotion fix (no duplicates).
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
