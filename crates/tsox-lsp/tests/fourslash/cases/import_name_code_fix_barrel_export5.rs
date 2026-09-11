use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_barrel_export5() {
    let content = r#"// @module: node18
// @Filename: /package.json
{ "type": "module" }
// @Filename: /foo/a.ts
export const A = 0;
// @Filename: /foo/b.ts
export {};
A/*sibling*/
// @Filename: /foo/index.ts
export * from "./a.js";
export * from "./b.js";
// @Filename: /index.ts
export * from "./foo/index.js";
export * from "./src/index.js";
// @Filename: /src/a.ts
export {};
A/*parent*/
// @Filename: /src/index.ts
export * from "./a.js";"#;
    let mut s = Session::new_for_test("importNameCodeFix_barrelExport5", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "sibling", []string{"./a.js", "./index.js", "../index.js"}, nil
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "parent", []string{"../foo/a.js", "../foo/index.js", "../index.
}
