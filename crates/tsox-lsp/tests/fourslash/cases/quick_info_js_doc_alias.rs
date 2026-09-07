use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_js_doc_alias() {
    let content = r#"// @filename: /a.d.ts
/** docs - type T */
export type T = () => void;
/**
 * docs - const A: T
 */
export declare const A: T;
// @filename: /b.ts
import { A } from "./a";
A/**/()"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
