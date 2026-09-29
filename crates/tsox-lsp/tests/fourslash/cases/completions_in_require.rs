use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_in_require() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
var foo = require("/**/"

foo();

/**
 * @return {void}
 */
function foo() {
}
// @Filename: package.json
 { "dependencies": { "fake-module": "latest" } }
// @Filename: node_modules/fake-module/index.js
/* fake-module */"#;
    let mut s = Session::new_for_test("completionsInRequire", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["fake-module"]);
}
