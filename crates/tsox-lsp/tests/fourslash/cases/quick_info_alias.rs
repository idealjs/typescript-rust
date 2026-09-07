use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_alias() {
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
