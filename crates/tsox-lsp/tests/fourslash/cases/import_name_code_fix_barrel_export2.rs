use tsox_lsp::fourslash::Session;


#[test]
fn import_name_code_fix_barrel_export2() {
    let content = r#"// @module: commonjs
// @baseUrl: /
// @Filename: /proj/foo/a.ts
export const A = 0;
// @Filename: /proj/foo/b.ts
export {};
A/*sibling*/
// @Filename: /proj/foo/index.ts
export * from "./a";
export * from "./b";
// @Filename: /proj/index.ts
export * from "./foo";
export * from "./src";
// @Filename: /proj/src/a.ts
export {};
A/*parent*/
// @Filename: /proj/src/utils.ts
export function util() { return "util"; }
export { A } from "../foo/a";
// @Filename: /proj/src/index.ts
export * from "./a";"#;
    let _s = Session::new_for_test("importNameCodeFix_barrelExport2", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "sibling", []string{"proj/foo/a", "proj/src/utils", "proj", "pr
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "parent", []string{"proj/foo", "proj/foo/a", "proj/src/utils", 
}
