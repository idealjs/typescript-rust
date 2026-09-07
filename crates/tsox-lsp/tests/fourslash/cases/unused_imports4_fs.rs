use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_imports4_fs() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
// @Filename: file2.ts
[| import {Calculator, test, test2} from "./file1" |]

var x = new Calculator();
x.handleChar();
test2();
// @Filename: file1.ts
export class Calculator {
    handleChar() {}
}

export function test() {

}

export function test2() {

}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `import {Calculator, test2} from "./file1"`, false, 0, 0)
}
