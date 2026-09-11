use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_fixes_global_typings_cache() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /project/tsconfig.json
 { "compilerOptions": { "allowJs": true, "checkJs": true, "module": "commonjs" } }
// @Filename: /home/src/Library/Caches/typescript/node_modules/@types/react-router-dom/package.json
 { "name": "@types/react-router-dom", "version": "16.8.4", "types": "index.d.ts" }
// @Filename: /home/src/Library/Caches/typescript/node_modules/@types/react-router-dom/index.d.ts
export class BrowserRouter {}
// @Filename: /project/node_modules/react-router-dom/package.json
 { "name": "react-router-dom", "version": "16.8.4", "main": "index.js" }
// @Filename: /project/node_modules/react-router-dom/index.js
 export const BrowserRouter = () => null;
// @Filename: /project/index.js
BrowserRouter/**/"#;
    let mut s = Session::new_for_test("importFixesGlobalTypingsCache", content);
    fourslash::go_to_file(&mut s, "/project/index.js");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
