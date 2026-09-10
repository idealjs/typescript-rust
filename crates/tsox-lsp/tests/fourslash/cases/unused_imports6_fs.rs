use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_imports6_fs() {
    // TODO: t.Skip("Known failing fourslash test")
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
    let mut s = Session::new_for_test("unusedImports6FS", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, ``, false, 0, 0)
}
