use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_imports1_fs() {
    let content = r#"// @noUnusedLocals: true
// @Filename: file2.ts
  [|import { Calculator } from "./file1" |]
// @Filename: file1.ts
   export class Calculator {

   }"#;
    let _s = Session::new_for_test("unusedImports1FS", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, ``, false, 0, 0)
}
