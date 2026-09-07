use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn javascript_modules_type_import() {
    let content = r#"// @allowJs: true
// @Filename: types.js
/**
 * @typedef {Object} Pet
 * @prop {string} name
 */
module.exports = { a: 1 };
// @Filename: app.js
/**
 * @param { import("./types")./**/ } p
 */
function walk(p) {
 console.log(` + "`" + `Walking ${p.name}...` + "`" + `);
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
