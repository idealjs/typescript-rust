use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_import_type6() {
    let content = r#"// @module: es2015
// @esModuleInterop: true
// @jsx: react
// @Filename: /types.d.ts
declare module "react" { var React: any; export = React; export as namespace React; }
// @Filename: /a.tsx
import type React from "react";
function Component() {}
(<Component/**/ />)"#;
    let mut s = Session::new_for_test("importNameCodeFix_importType6", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
