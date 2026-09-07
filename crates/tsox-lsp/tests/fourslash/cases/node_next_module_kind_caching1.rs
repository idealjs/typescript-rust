use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn node_next_module_kind_caching1() {
    let content = r#"// @Filename: tsconfig.json
{
    "compilerOptions": {
      "lib": ["es5"],
      "rootDir": "src",
      "outDir": "dist",
      "target": "ES2020",
      "module": "NodeNext",
      "strict": true
    },
    "include": ["src\\**\\*.ts"]
}
// @Filename: package.json
{
    "type": "module",
    "private": true
}
// @Filename: src/index.ts
// The line below should show a "Relative import paths need explicit file
// extensions..." error in VS Code, but it doesn't. The error is only picked up
// by ` + "`" + `tsc` + "`" + ` which seems to properly infer the module type.
import { helloWorld } from './example'
/**/
helloWorld()
// @Filename: src/example.ts
export function helloWorld() {
    console.log('Hello, world!')
}"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
