use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_string_literal_nonrelative_import12() {
    let content = r#"// @Filename: tests/test0.ts
import * as foo1 from "m/*import_as0*/
import foo2 = require("m/*import_equals0*/
var foo3 = require("m/*require0*/
// @Filename: package.json
{
    "dependencies": { "module": "latest" },
    "devDependencies": { "dev-module": "latest" },
    "optionalDependencies": { "optional-module": "latest" },
    "peerDependencies": { "peer-module": "latest" }
}"#;
    let mut s = Session::new_for_test("completionForStringLiteralNonrelativeImport12", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
