use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_js_module_name_at_import_name() {
    let content = r#"// @allowJs: true
// @Filename: /foo.js
 /*moduleDef*/function notExported() { }
 class Blah {
    abc = 123;
 }
 module.exports.Blah = Blah;
// @Filename: /bar.js
const [|/*importDef*/BlahModule|] = require("./foo.js");
new [|/*importUsage*/BlahModule|].Blah()
// @Filename: /barTs.ts
import [|/*importDefTs*/BlahModule|] = require("./foo.js");
new [|/*importUsageTs*/BlahModule|].Blah()"#;
    let _s = Session::new_for_test("goToDefinitionJsModuleNameAtImportName", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "importDef", "importUsage", "importDefTs", "importUsageTs")
}
