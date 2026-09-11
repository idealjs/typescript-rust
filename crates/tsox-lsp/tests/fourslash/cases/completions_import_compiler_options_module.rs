use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_compiler_options_module() {
    let content = r#"// @allowJs: true
// @module: commonjs
// @Filename: /node_modules/a/index.d.ts
export const foo = 0;
// @Filename: /b.js
const a = require("./a");
fo/*b*/
// @Filename: /c.js
const x = 0;/*c*/
// @Filename: /c1.js
// @ts-check
const x = 0;/*ccheck*/
// @Filename: /c2.ts
const x = 0;/*cts*/
// @Filename: /d.js
const a = import("./a"); // Does not make this an external module
fo/*d*/
// @Filename: /d1.js
// @ts-check
const a = import("./a"); // Does not make this an external module
fo/*dcheck*/
// @Filename: /d2.ts
const a = import("./a"); // Does not make this an external module
fo/*dts*/"#;
    let mut s = Session::new_for_test("completionsImport_compilerOptionsModule", content);
    // TODO: f.VerifyCompletions(t, []string{"b", "c", "ccheck", "cts", "d", "dcheck", "dts"}, &fourslash.Complet
}
