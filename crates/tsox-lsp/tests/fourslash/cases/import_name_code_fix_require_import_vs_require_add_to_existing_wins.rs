use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_require_import_vs_require_add_to_existing_wins() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: blah.js
export default class Blah {}
export const Named1 = 0;
export const Named2 = 1;
// @Filename: index.js
var path = require('path')
  , { promisify } = require('util')
  , { Named1 } = require('./blah')

import fs from 'fs'

new Blah"#;
    let mut s = Session::new_for_test("importNameCodeFix_require_importVsRequire_addToExistingWins", content);
    fourslash::go_to_file(&mut s, "index.js");
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
