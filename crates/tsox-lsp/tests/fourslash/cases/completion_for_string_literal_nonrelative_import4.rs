use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_string_literal_nonrelative_import4() {
    let content = r#"// @Filename: dir1/dir2/dir3/dir4/test0.ts
import * as foo1 from "f/*import_as0*/
import foo4 = require("f/*import_equals0*/
var foo7 = require("f/*require0*/
// @Filename: package.json
{ "dependencies": { "fake-module": "latest" } }
// @Filename: node_modules/fake-module/ts.ts

// @Filename: dir1/package.json
{ "dependencies": { "fake-module2": "latest" } }
// @Filename: dir1/node_modules/fake-module2/index.ts

// @Filename: dir1/dir2/dir3/package.json
{ "dependencies": { "fake-module3": "latest" } }
// @Filename: dir1/dir2/dir3/node_modules/fake-module3/ts.ts
"#;
    let mut s = Session::new_for_test("completionForStringLiteralNonrelativeImport4", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
