use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_global() {
    let content = r#"// @Filename: /src/globals.d.ts
export {}; // Make this a module
declare global {
    interface Disposable {
        [Symbol.dispose](): void;
    }
}
// @Filename: /src/test.ts
import { Service } from './lifecycle';
export class [|EditingService|] implements Service { }
// @Filename: /src/lifecycle.ts
export interface Disposable {
	(): string;
}
export interface Service {
	d: Disposable;
}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceGlobal", content);
    fourslash::go_to_file(&mut s, "/src/test.ts");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
