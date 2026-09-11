use tsox_lsp::fourslash::{self, Session};


#[test]
fn unused_imports13_fs() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
// @Filename: file2.ts
[| import A, { x } from './a'; |]
console.log(A);
// @Filename: file1.ts
export default 10;
export var x = 10;"#;
    let mut s = Session::new_for_test("unusedImports13FS", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `import A from './a';`, false, 0, 0)
}
