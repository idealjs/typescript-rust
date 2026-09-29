use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_convert_to_type_only_import1() {
    let content = r#"// @module: esnext
// @verbatimModuleSyntax: true
// @Filename: exports.ts
export default class A {}
export class B {}
export class C {}
// @Filename: imports.ts
import {
    B,
    C,
} from './exports';

declare const b: B;
declare const c: C;
console.log(b, c);"#;
    let mut s = Session::new_for_test("codeFixConvertToTypeOnlyImport1", content);
    fourslash::go_to_file(&mut s, "imports.ts");
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
