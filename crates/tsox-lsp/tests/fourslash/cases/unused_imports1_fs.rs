use tsox_lsp::fourslash::{self, Session};


#[test]
fn unused_imports1_fs() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
// @Filename: file2.ts
  [|import { Calculator } from "./file1" |]
// @Filename: file1.ts
   export class Calculator {

   }"#;
    let mut s = Session::new_for_test("unusedImports1FS", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, ``, false, 0, 0)
}
