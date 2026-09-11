use tsox_lsp::fourslash::{self, Session};


#[test]
fn unused_imports7_fs() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
// @Filename: file2.ts
[| import * as n from "./file1" |]
// @Filename: file1.ts
export class Calculator {
    handleChar() { }
}
export function test() {
}
export default function test2() {
}"#;
    let mut s = Session::new_for_test("unusedImports7FS", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, ``, false, 0, 0)
}
