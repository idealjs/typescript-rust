use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_imports10_fs() {
    let content = r#"// @noUnusedLocals: true
namespace A {
   export class Calculator {
        public handelChar() {
        }
    }
}
namespace B {
    [|import a = A;|]
}"#;
    let _s = Session::new_for_test("unusedImports10FS", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, ``, false, 0, 0)
}
