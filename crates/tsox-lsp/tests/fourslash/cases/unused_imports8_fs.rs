use tsox_lsp::fourslash::{self, Session};


#[test]
fn unused_imports8_fs() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
// @Filename: file2.ts
[|import {Calculator as calc, test as t1, test2 as t2} from "./file1"|]

var x = new calc();
x.handleChar();
t1();
// @Filename: file1.ts
export class Calculator {
    handleChar() { }
}
export function test() {

}
export function test2() {

}"#;
    let mut s = Session::new_for_test("unusedImports8FS", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `import {Calculator as calc, test as t1} from "./file1"`, false, 0, 0)
}
