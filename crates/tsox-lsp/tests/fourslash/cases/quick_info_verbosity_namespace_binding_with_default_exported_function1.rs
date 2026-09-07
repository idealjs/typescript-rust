use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quick_info_verbosity_namespace_binding_with_default_exported_function1() {
    let content = r#"// @module: esnext
// @filename: /a.ts
export default function fn() {}
export { fn as default };
// @filename: /b.ts
import * as ns from "./a";

ns/*1*/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
