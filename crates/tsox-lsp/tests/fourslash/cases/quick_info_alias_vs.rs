use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineVSHover"]
#[test]
fn quick_info_alias_vs() {
    let content = r#"// @Filename: /a.ts
/**
 * Doc
 * @tag Tag text
 */
export const x = 0;
// @Filename: /b.ts
import { x } from "./a";
x/*b*/;
// @Filename: /c.ts
/**
 * Doc 2
 * @tag Tag text 2
 */
import {
    /**
     * Doc 3
     * @tag Tag text 3
     */
    x
} from "./a";
x/*c*/;"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyBaselineVSHover"); // f.VerifyBaselineVSHover(t)
}
