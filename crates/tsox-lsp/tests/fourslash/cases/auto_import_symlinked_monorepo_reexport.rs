use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // This should show projectBFunction once via re-export and "]
#[test]
fn auto_import_symlinked_monorepo_reexport() {
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
// @Filename: /packages/project-b/src/utils/foo.ts
export function projectBFunction(): string { return "hello"; }
// @Filename: /packages/project-b/src/index.ts
export * from './utils/foo';
export const projectBValue: number = 42;
// @Filename: /packages/project-b/dist/utils/foo.d.ts
export declare function projectBFunction(): string;
// @Filename: /packages/project-b/dist/index.d.ts
export * from './utils/foo';
export declare const projectBValue: number;
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
projectBFunction/**/
// @link: /packages/project-b -> /packages/project-a/node_modules/project-b"#;
    let mut s = Session::new(content);
    // TODO: prefs := lsutil.NewDefaultUserPreferences()
    // TODO: prefs.AutoImportEntrypointDirectorySearch = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, prefs)
    fourslash::go_to_marker(&mut s, "");
    // TODO: // This should show projectBFunction once via re-export and once via direct import, not duplicates
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
