use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // This should show projectBFunction once, not twice (not fr"]
#[test]
fn auto_import_symlinked_monorepo_project_references_no_pkg_exports() {
    let content = r#"// @Filename: /packages/project-b/tsconfig.json
{
  "compilerOptions": {
    "composite": true,
    "outDir": "./dist",
    "rootDir": "./src",
    "declaration": true,
    "module": "commonjs",
    "strict": true
  },
  "include": ["src"]
}
// @Filename: /packages/project-b/package.json
{
  "name": "project-b",
  "version": "1.0.0",
  "main": "dist/index.js",
  "types": "dist/index.d.ts"
}
// @Filename: /packages/project-b/src/index.ts
export const projectBValue: number = 42;
export function projectBFunction(): string { return "hello"; }
// @Filename: /packages/project-b/dist/index.d.ts
export declare const projectBValue: number;
export declare function projectBFunction(): string;
// @Filename: /packages/project-a/tsconfig.json
{
  "compilerOptions": {
    "module": "commonjs",
    "strict": true,
    "outDir": "./dist",
    "rootDir": "./src"
  },
  "include": ["src"],
  "references": [{ "path": "../project-b" }]
}
// @Filename: /packages/project-a/package.json
{ "name": "project-a", "dependencies": { "project-b": "*" } }
// @Filename: /packages/project-a/src/index.ts
import { projectBValue } from "project-b";
console.log(projectBValue);
projectBFunc/**/
// @link: /packages/project-b -> /packages/project-a/node_modules/project-b"#;
    let mut s = Session::new_for_test("autoImportSymlinkedMonorepoProjectReferencesNoPkgExports", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: // This should show projectBFunction once, not twice (not from both src and dist)
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}
