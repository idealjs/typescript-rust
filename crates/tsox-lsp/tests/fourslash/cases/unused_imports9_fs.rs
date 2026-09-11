use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_imports9_fs() {
    let content = r#"// @noUnusedLocals: true
// @Filename: file2.ts
[|import c = require('./file1')|]
// @Filename: file1.ts
export class Calculator {
    handleChar() { }
}

export function test() {

}

export function test2() {

}"#;
    let mut s = Session::new_for_test("unusedImports9FS", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, ``, false, 0, 0)
}
