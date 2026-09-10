use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_imports3_fs() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
// @Filename: file2.ts
[| import {Calculator, /*some comments*/ test, test2} from "./file1" |]
 test();
 test2();
// @Filename: file1.ts
 export class Calculator {
     handleChar() {}
 }
 export function test() {

 }
 export function test2() {

 }"#;
    let mut s = Session::new_for_test("unusedImports3FS", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `import {/*some comments*/ test, test2} from "./file1"`, false, 0, 0)
}
