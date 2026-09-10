use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports15() {
    let content = r#"// @filename: /a.ts
export const foo = 1;
// @filename: /b.ts
/**
 * Module doc comment
 *
 * @module
 */

// comment 1

// comment 2

import { foo } from "./a";"#;
    let mut s = Session::new_for_test("organizeImports15", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
