use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_imports10_fs() {
    // TODO: t.Skip("Known failing fourslash test")
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, ``, false, 0, 0)
}
