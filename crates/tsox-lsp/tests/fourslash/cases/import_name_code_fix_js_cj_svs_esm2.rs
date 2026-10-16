use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_js_cj_svs_esm2() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: types/dep.d.ts
export declare class Dep {}
// @Filename: index.js
Dep/**/
// @Filename: util1.ts
import fs from 'fs';
// @Filename: util2.js
const fs = require('fs');"#;
    let mut s = Session::new_for_test("importNameCodeFix_jsCJSvsESM2", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
