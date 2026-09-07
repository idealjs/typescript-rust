use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_import_promote_type_only2() {
    let content = r#"// @module: es2015
// @Filename: /exports.ts
export interface SomeInterface {}
// @Filename: /a.ts
import type { SomeInterface } from "./exports.js";
const SomeInterface = {};
SomeI/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
