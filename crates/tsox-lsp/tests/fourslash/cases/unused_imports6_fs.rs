use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_imports6_fs() {
    let content = r#"// @noUnusedLocals: true
// @Filename: file2.ts
[| import d from "./file1" |]
// @Filename: file1.ts
export class Calculator {
    handleChar() { }
}
export function test() {

}
export default function test2() {

}"#;
    let _s = Session::new_for_test("unusedImports6FS", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, ``, false, 0, 0)
}
