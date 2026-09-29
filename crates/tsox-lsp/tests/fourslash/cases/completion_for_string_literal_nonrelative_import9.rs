use tsox_lsp::fourslash::Session;


#[test]
fn completion_for_string_literal_nonrelative_import9() {
    let content = r#"// @Filename: tsconfig.json
{
    "compilerOptions": {
        "baseUrl": "./modules",
        "paths": {
            "module1": ["some/path/whatever.ts"],
            "module2": ["some/other/path.ts"]
        }
    }
}
// @Filename: tests/test0.ts
import * as foo1 from "m/*import_as0*/
import foo2 = require("m/*import_equals0*/
var foo3 = require("m/*require0*/
// @Filename: some/path/whatever.ts
export var x = 9;
// @Filename: some/other/path.ts
export var y = 10;"#;
    let _s = Session::new_for_test("completionForStringLiteralNonrelativeImport9", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
