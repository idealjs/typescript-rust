use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_default_re_export() {
    let content = r#"// @lib: es5
// @module: commonjs
// @allowJs: true
// @Filename: /file1.js
const a = 1;
export {
    a as b
};
export default a;
// @Filename: /file2.js
import * as foo from './file1';
/**/
export default foo.b;"#;
    let mut s = Session::new_for_test("completionsImport_default_reExport", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
