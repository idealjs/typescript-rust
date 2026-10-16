use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_type_in_other_file() {
    let content = r#"// @Filename: /I.ts
export interface J {}
export interface I {
    x: J;
    m(): J;
}
// @Filename: /C.ts
import { I } from "./I";
export class C implements I {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterface_typeInOtherFile", content);
    fourslash::go_to_file(&mut s, "/C.ts");
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
