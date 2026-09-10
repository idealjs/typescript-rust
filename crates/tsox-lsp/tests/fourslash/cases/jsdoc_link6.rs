use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn jsdoc_link6() {
    let content = r#"// @filename: /a.ts
export default function A() { }
export function B() { };
// @Filename: /b.ts
import A, { B } from "./a";
/**
 * {@link A}
 * {@link B}
 */
export default function /**/f() { }"#;
    let mut s = Session::new_for_test("jsdocLink6", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
