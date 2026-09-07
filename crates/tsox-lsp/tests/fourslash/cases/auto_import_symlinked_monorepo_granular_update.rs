use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // At this point, newlyAddedFunction doesn't exist yet in pr"]
#[test]
fn auto_import_symlinked_monorepo_granular_update() {
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
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "default": "./dist/index.js"
    }
  }
}
// @Filename: /packages/project-b/src/index.ts
export const projectBValue: number = 42;
/*projectBEdit*/
// @Filename: /packages/project-b/dist/index.d.ts
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
newlyAdded/*projectACompletion*/
// @link: /packages/project-b -> /packages/project-a/node_modules/project-b"#;
    let mut s = Session::new(content);
    // TODO: // Get initial completions in project-a - this builds the initial auto-import index.
    // TODO: // At this point, newlyAddedFunction doesn't exist yet in project-b.
    fourslash::go_to_marker(&mut s, "projectACompletion");
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{"projectACompletion"})
    // TODO: // Now edit project-b's source file to add a new export.
    // TODO: // This should trigger a granular update when we request completions again.
    fourslash::go_to_marker(&mut s, "projectBEdit");
    fourslash::insert(&mut s, "\nexport function newlyAddedFunction(): void {}");
    // TODO: // Go back to project-a and request completions again.
    // TODO: // The granular update should have picked up the new export from project-b.
    fourslash::go_to_marker(&mut s, "projectACompletion");
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{"projectACompletion"})
}
