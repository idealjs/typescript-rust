use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_auto_imports_re_exports() {
    let content = r#"// @Filename: node_modules/test-module/index.d.ts
declare namespace e {
    interface Foo {}
}
export = e;
// @Filename: a.ts
import { Foo } from "test-module";
export interface A {
    foo(): Foo;
}
// @Filename: b.ts
import { A } from "./a";
export class B implements A {}"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "b.ts");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
