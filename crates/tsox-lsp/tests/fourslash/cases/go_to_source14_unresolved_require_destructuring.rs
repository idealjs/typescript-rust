use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn go_to_source14_unresolved_require_destructuring() {
    let content = r#"// @lib: es5
// @allowJs: true
// @Filename: /home/src/workspaces/project/index.js
const { blah/**/ } = require("unresolved");"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "")
}
