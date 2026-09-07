use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_provider_global_typings_cache() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /home/src/Library/Caches/typescript/node_modules/@types/react-router-dom/package.json
 { "name": "@types/react-router-dom", "version": "16.8.4", "types": "index.d.ts" }
// @Filename: /home/src/Library/Caches/typescript/node_modules/@types/react-router-dom/index.d.ts
 export class BrowserRouterFromDts {}
// @Filename: /home/src/workspaces/project/package.json
 { "dependencies": { "react-router-dom": "*" } }
// @Filename: /home/src/workspaces/project/tsconfig.json
 { "compilerOptions": { "module": "commonjs", "lib": ["es5"], "allowJs": true, "checkJs": true, "maxNodeModuleJsDepth": 2 }, "typeAcquisition": { "enable": true } }
// @Filename: /home/src/workspaces/project/node_modules/react-router-dom/package.json
 { "name": "react-router-dom", "version": "16.8.4", "main": "index.js" }
// @Filename: /home/src/workspaces/project/node_modules/react-router-dom/index.js
 import "./BrowserRouter";
 export {};
// @Filename: /home/src/workspaces/project/node_modules/react-router-dom/BrowserRouter.js
 export const BrowserRouterFromJs = () => null;
// @Filename: /home/src/workspaces/project/index.js
BrowserRouter/**/"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
