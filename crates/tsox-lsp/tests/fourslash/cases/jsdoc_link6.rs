use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("jsdocLink6", content);
    // TODO: f.VerifyBaselineHover(t)
}
