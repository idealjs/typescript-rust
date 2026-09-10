use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_auto_imports_type_only() {
    let content = r#"// @module: esnext
// @verbatimModuleSyntax: true
// @Filename: types1.ts
type A = {};
export default A;
// @Filename: types2.ts
export type B = {};
export type C = {};
export type D<T> = {};
// @Filename: interface.ts
import type A from './types1';
import type { B, C, D } from './types2';

export interface Base {
  a: A;
  b<T extends B = B>(p1: C): D<C>;
}
// @Filename: index.ts
import type { Base } from './interface';

export class C implements Base {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceAutoImports_typeOnly", content);
    fourslash::go_to_file(&mut s, "index.ts");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
