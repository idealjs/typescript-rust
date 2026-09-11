use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_promote_type_only2() {
    let content = r#"// @module: es2015
// @Filename: /exports.ts
export interface SomeInterface {}
// @Filename: /a.ts
import type { SomeInterface } from "./exports.js";
const SomeInterface = {};
SomeI/**/"#;
    let mut s = Session::new_for_test("completionsImport_promoteTypeOnly2", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
