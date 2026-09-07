use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn hover_circular_inherited_documentation() {
    let content = r#"// @filename: base.ts
export interface Options {}
// @filename: bridge.ts
import type { Options as _Options } from "./base";
export * from "./base";
declare module "./bridge" {
    interface Options extends _Options { hooks: {} }
}
declare const v: Options;
v.hooks/*1*/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) Options.hooks: {}", "")
}
