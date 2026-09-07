use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_quote_preference_auto2() {
    let content = r#"// @filename: a.ts
export interface I {
    a(): void;
    b(x: 'x', y: 'a' | 'b'): 'b';

    c: 'c';
    d: { e: 'e'; };
}
// @filename: b.ts
import { I } from './a';
class Foo implements I {}"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "b.ts");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
