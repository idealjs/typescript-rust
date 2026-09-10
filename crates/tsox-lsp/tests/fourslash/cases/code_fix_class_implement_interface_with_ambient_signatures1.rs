use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_with_ambient_signatures1() {
    let content = r#"// @lib: esnext
// @target: esnext
// @Filename: /node_modules/@types/node/globals.d.ts
export {};
declare global {
    interface SymbolConstructor {
        readonly dispose: unique symbol;
    }
    interface Disposable {
        [Symbol.dispose](): void;
    }
}
// @Filename: /node_modules/@types/node/index.d.ts
/// <reference path="globals.d.ts" />
// @Filename: a.ts
class Foo implements Disposable {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceWithAmbientSignatures1", content);
    fourslash::go_to_file(&mut s, "a.ts");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
