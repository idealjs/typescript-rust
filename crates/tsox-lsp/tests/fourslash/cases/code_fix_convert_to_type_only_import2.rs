use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_convert_to_type_only_import2() {
    let content = r#"// @module: esnext
// @verbatimModuleSyntax: true
// @Filename: exports.ts
export default class A {}
export class B {}
export class C {}
// @Filename: imports.ts
import A, { B, C } from './exports';

declare const a: A;
declare const b: B;
declare const c: C;
console.log(a, b, c);"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "imports.ts");
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
