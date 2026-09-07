use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
