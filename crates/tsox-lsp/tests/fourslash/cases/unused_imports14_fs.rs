use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_imports14_fs() {
    let content = r#"// @noUnusedLocals: true
// @Filename: file2.ts
[| import /* 1 */ A /* 2 */, /* 3 */ { /* 4 */ x /* 5 */ } /* 6 */ from './a'; |]
console.log(A);
// @Filename: file1.ts
export default 10;
export var x = 10;"#;
    let mut s = Session::new_for_test("unusedImports14FS", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `import /* 1 */ A /* 2 */ /* 6 */ from './a';`, false, 0, 0)
}
