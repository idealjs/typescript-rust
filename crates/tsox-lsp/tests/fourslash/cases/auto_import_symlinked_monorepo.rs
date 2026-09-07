use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // This should show projectBFunction once, not twice"]
#[test]
fn auto_import_symlinked_monorepo() {
    let content = r#"// @Filename: /packages/project-b/package.json
{ "name": "project-b", "version": "1.0.0", "main": "index.js", "types": "index.d.ts" }
// @Filename: /packages/project-b/index.d.ts
export declare const projectBValue: number;
export declare function projectBFunction(): string;
// @Filename: /packages/project-a/tsconfig.json
{ "compilerOptions": { "module": "commonjs", "strict": true } }
// @Filename: /packages/project-a/package.json
{ "name": "project-a", "dependencies": { "project-b": "*" } }
// @Filename: /packages/project-a/index.ts
import { projectBValue } from "project-b";
console.log(projectBValue);
projectBFunc/**/
// @link: /packages/project-b -> /packages/project-a/node_modules/project-b"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: // This should show projectBFunction once, not twice
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}
