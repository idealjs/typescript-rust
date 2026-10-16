use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_imports11_fs() {
    let content = r#"// @noUnusedLocals: true
// @Filename: file2.ts
[| import f1, * as s from "./file1"; |]
s.f2('hello');
// @Filename: file1.ts
export var v1;
export function f1(n: number){}
export function f2(s: string){};
export default f1;"#;
    let _s = Session::new_for_test("unusedImports11FS", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `import * as s from "./file1";`, false, 0, 0)
}
