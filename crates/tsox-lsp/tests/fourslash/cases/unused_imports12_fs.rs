use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_imports12_fs() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
// @Filename: file2.ts
[| import f1, * as s from "./file1"; |]
f1(42);
// @Filename: file1.ts
export function f1(n: number){}
export function f2(s: string){};
export default f1;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `import f1 from "./file1";`, false, 0, 0)
}
