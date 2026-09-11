use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_import_type3() {
    let content = r#"// @verbatimModuleSyntax: true
// @module: es2015
// @Filename: /exports.ts
class SomeClass {}
export type { SomeClass };
// @Filename: /a.ts
import {} from "./exports.js";
function takeSomeClass(c: SomeClass/**/)"#;
    let mut s = Session::new_for_test("importNameCodeFix_importType3", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
