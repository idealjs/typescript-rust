use tsox_lsp::fourslash::Session;


#[test]
fn completion_for_string_literal_nonrelative_import_typings1() {
    let content = r#"// @typeRoots: my_typings,my_other_typings
// @Filename: tests/test0.ts
/// <reference types="m/*types_ref0*/" />
import * as foo1 from "m/*import_as0*/
import foo2 = require("m/*import_equals0*/
var foo3 = require("m/*require0*/
// @Filename: my_typings/module-x/index.d.ts
export var x = 9;
// @Filename: my_typings/module-x/whatever.d.ts
export var w = 9;
// @Filename: my_typings/module-y/index.d.ts
export var y = 9;
// @Filename: my_other_typings/module-z/index.d.ts
export var z = 9;"#;
    let _s = Session::new_for_test("completionForStringLiteralNonrelativeImportTypings1", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
