use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn javascript_modules_type_import_as_value() {
    let content = r#"// @allowJs: true
// @Filename: types.js
/**
 * @typedef {Object} Pet
 * @prop {string} name
 */
module.exports = { a: 1 };
// @Filename: app.js
import { /**/ } from "./types""#;
    let mut s = Session::new_for_test("javascriptModulesTypeImportAsValue", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
