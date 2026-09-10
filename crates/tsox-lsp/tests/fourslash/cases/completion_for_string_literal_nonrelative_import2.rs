use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_string_literal_nonrelative_import2() {
    let content = r#"// @Filename: tests/test0.ts
import * as foo1 from "fake-module//*import_as0*/
import foo2 = require("fake-module//*import_equals0*/
var foo3 = require("fake-module//*require0*/
// @Filename: package.json
{ "dependencies": { "fake-module": "latest" }, "devDependencies": { "fake-module-dev": "latest" } }
// @Filename: node_modules/fake-module/repeated.ts
/*repeatedts*/
// @Filename: node_modules/fake-module/repeated.tsx
/*repeatedtsx*/
// @Filename: node_modules/fake-module/repeated.d.ts
/*repeateddts*/
// @Filename: node_modules/fake-module/other.js
/*other*/
// @Filename: node_modules/fake-module/other2.js
/*other2*/
// @Filename: node_modules/unlisted-module/index.js
/*unlisted-module*/
// @Filename: ambient.ts
declare module "fake-module/other""#;
    let mut s = Session::new_for_test("completionForStringLiteralNonrelativeImport2", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"import_as0", "import_equals0", "require0"}, &fourslash.CompletionsE
}
