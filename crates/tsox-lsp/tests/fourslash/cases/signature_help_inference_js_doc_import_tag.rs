use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn signature_help_inference_js_doc_import_tag() {
    let content = r#"// @allowJS: true
// @checkJs: true
// @module: esnext
// @filename: a.ts
export interface Foo {}
// @filename: b.js
/**
 * @import {
 *     Foo
 * } from './a'
 */

/**
 * @param {Foo} a
 */
function foo(a) {}
foo(/**/)"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
