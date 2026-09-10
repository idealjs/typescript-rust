use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports10() {
    let content = r#"// @Filename: /module.ts
import type { ZodType } from './declaration';

/** Intended to be used in combination with {@link ZodType} */
export function fun() { /* ... */ }
// @Filename: /declaration.ts
 type ZodType = {};
 export type { ZodType }"#;
    let mut s = Session::new_for_test("organizeImports10", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
