use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn import_name_code_fix_require_import_vs_require_import_wins() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: blah.js
export default class Blah {}
export const Named1 = 0;
export const Named2 = 1;
// @Filename: addToExisting.js
const { Named2 } = require('./blah')
import { Named1 } from './blah'

new Blah
// @Filename: newImport.js
import fs from 'fs';
const path = require('path');

new Blah"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "addToExisting.js");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
    fourslash::go_to_file(&mut s, "newImport.js");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
