use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_trailing_comma() {
    let content = r#"// @Filename: index.ts
import {
  T2,
  T1,
} from "./types";

const x: T3/**/
// @Filename: types.ts
export type T1 = 0;
export type T2 = 0;
export type T3 = 0;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
