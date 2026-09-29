use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_imports3_fs() {
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
    let _s = Session::new_for_test("unusedImports3FS", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `import {/*some comments*/ test, test2} from "./file1"`, false, 0, 0)
}
