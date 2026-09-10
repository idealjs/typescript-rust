use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_import_re_export_default2() {
    let content = r#"// @lib: es5
// @module: preserve
// @checkJs: true
// @Filename: /node_modules/example/package.json
{ "name": "example", "version": "1.0.0", "main": "dist/index.js" }
// @Filename: /node_modules/example/dist/nested/module.d.ts
declare const defaultExport: () => void;
declare const namedExport: () => void;

export default defaultExport;
export { namedExport };
// @Filename: /node_modules/example/dist/index.d.ts
export { default, namedExport } from "./nested/module";
// @Filename: /index.mjs
import { namedExport } from "example";
defaultExp/**/"#;
    let mut s = Session::new_for_test("completionsImport_reExportDefault2", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
