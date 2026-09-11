use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_source14_unresolved_require_destructuring() {
    let content = r#"// @lib: es5
// @allowJs: true
// @Filename: /home/src/workspaces/project/index.js
const { blah/**/ } = require("unresolved");"#;
    let mut s = Session::new_for_test("goToSource14_unresolvedRequireDestructuring", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "")
}
