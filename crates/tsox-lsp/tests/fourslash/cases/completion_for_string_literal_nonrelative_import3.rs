use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_string_literal_nonrelative_import3() {
    let content = r#"// @allowJs: true
// @Filename: tests/test0.ts
import * as foo1 from "fake-module//*import_as0*/
import foo2 = require("fake-module//*import_equals0*/
var foo3 = require("fake-module//*require0*/
// @Filename: package.json
{ "dependencies": { "fake-module": "latest" } }
// @Filename: node_modules/fake-module/ts.ts
/*ts*/
// @Filename: node_modules/fake-module/tsx.tsx
/*tsx*/
// @Filename: node_modules/fake-module/dts.d.ts
/*dts*/
// @Filename: node_modules/fake-module/js.js
/*js*/
// @Filename: node_modules/fake-module/jsx.jsx
/*jsx*/
// @Filename: node_modules/fake-module/repeated.js
/*repeatedjs*/
// @Filename: node_modules/fake-module/repeated.jsx
/*repeatedjsx*/"#;
    let mut s = Session::new_for_test("completionForStringLiteralNonrelativeImport3", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"import_as0", "import_equals0", "require0"}, &fourslash.CompletionsE
}
