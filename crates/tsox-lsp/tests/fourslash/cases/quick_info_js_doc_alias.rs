use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("quickInfoJsDocAlias", content);
    // TODO: f.VerifyBaselineHover(t)
}
