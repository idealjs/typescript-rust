use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixAll"]
#[test]
fn import_name_code_fix_all_promote_type() {
    let content = r#"// @Filename: /a.ts
export class A {}
export class B {}
export class C {}
export class D {}
export class E {}
export class F {}
export class G {}
// @Filename: /b.ts
import type { A, C, D, E, G } from './a';
type Z = B | A;
new F;
// @Filename: /c.ts
import type { A, C, D, E, G } from './a';
type Z = B | A;
type Y = F;"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
    fourslash::go_to_file(&mut s, "/c.ts");
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
