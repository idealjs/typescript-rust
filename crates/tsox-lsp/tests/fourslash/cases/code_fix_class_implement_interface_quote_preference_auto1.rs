use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_quote_preference_auto1() {
    let content = r#"// @filename: a.ts
export interface I {
    a(): void;
    b(x: "x", y: "a" | "b"): "b";

    c: "c";
    d: { e: "e"; };
}
// @filename: b.ts
import { I } from "./a";
class Foo implements I {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterface_quotePreferenceAuto1", content);
    fourslash::go_to_file(&mut s, "b.ts");
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
