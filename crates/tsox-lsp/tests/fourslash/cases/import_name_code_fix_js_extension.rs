use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_js_extension() {
    let content = r#"// @moduleResolution: bundler
// @noLib: true
// @jsx: preserve
// @Filename: /a.ts
export function a() {}
// @Filename: /b.ts
export function b() {}
// @Filename: /c.tsx
export function c() {}
// @Filename: /c.ts
import * as g from "global"; // Global imports skipped
import { a } from "./a.js";
import { a as a2 } from "./a"; // Ignored, only the first relative import is considered
b; c;"#;
    let mut s = Session::new_for_test("importNameCodeFix_jsExtension", content);
    fourslash::go_to_file(&mut s, "/c.ts");
    // TODO: f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
