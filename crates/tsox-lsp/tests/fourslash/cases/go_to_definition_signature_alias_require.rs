use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_signature_alias_require() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
module.exports = function /*f*/f() {}
// @Filename: /b.js
const f = require("./a");
[|/*use*/f|]();
// @Filename: /bar.ts
import f = require("./a");
[|/*useTs*/f|]();"#;
    let _s = Session::new_for_test("goToDefinitionSignatureAlias_require", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "use", "useTs")
}
