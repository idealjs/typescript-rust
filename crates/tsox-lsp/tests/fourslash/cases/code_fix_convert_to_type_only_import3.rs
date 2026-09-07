use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_convert_to_type_only_import3() {
    let content = r#"// @module: esnext
// @verbatimModuleSyntax: true
// @Filename: exports1.ts
export default class A {}
export class B {}
export class C {}
// @Filename: exports2.ts
export default class D {}
export class E {}
export class F {}
// @Filename: imports.ts
import A, { B, C } from './exports1';
import D, * as others from "./exports2";

declare const a: A;
declare const b: B;
declare const c: C;
declare const d: D;
declare const o: typeof others;
console.log(a, b, c, d, o);"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "imports.ts");
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
