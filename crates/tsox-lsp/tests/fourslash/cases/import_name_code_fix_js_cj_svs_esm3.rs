use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_js_cj_svs_esm3() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: types/dep.d.ts
export declare class Dep {}
// @Filename: index.js
import fs from 'fs';
const path = require('path');

Dep/**/
// @Filename: util2.js
export {};"#;
    let mut s = Session::new_for_test("importNameCodeFix_jsCJSvsESM3", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
