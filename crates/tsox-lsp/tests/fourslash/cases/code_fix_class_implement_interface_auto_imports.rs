use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_auto_imports() {
    let content = r#"// @Filename: types1.ts
type A = {};
export default A;
// @Filename: types2.ts
export type B = {};
export type C = {};
export type D<T> = {};
// @Filename: interface.ts
import A from './types1';
import { B, C, D } from './types2';

export interface Base {
  a: Readonly<A> & { kind: "a"; };
  b<T extends B = B>(p1: C): D<C>;
}
// @Filename: index.ts
import { Base } from './interface';

export class C implements Base {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceAutoImports", content);
    fourslash::go_to_file(&mut s, "index.ts");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
