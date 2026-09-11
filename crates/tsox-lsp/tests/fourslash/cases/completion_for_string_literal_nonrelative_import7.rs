use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal_nonrelative_import7() {
    let content = r#"// @baseUrl: tests/cases/fourslash/modules
// @Filename: tests/test0.ts
import * as foo1 from "mod/*import_as0*/
import foo2 = require("mod/*import_equals0*/
var foo3 = require("mod/*require0*/
// @Filename: modules/module.ts
export var x = 5;
// @Filename: package.json
{ "dependencies": { "module-from-node": "latest" } }
// @Filename: node_modules/module-from-node/index.ts
"#;
    let mut s = Session::new_for_test("completionForStringLiteralNonrelativeImport7", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
