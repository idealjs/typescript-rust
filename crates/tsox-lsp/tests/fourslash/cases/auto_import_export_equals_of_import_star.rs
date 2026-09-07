use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
#[test]
fn auto_import_export_equals_of_import_star() {
    let content = r#"// @module: commonjs
// @Filename: /node_modules/mdx/package.json
{ "name": "mdx", "version": "1.0.0", "types": "index.d.ts" }
// @Filename: /node_modules/mdx/index.d.ts
import * as mdx from './lib/index.js'

export = mdx
// @Filename: /node_modules/mdx/lib/index.d.ts
export * from './core.js'
export * from './compile.js'
// @Filename: /node_modules/mdx/lib/core.d.ts
export declare function core(): void
// @Filename: /node_modules/mdx/lib/compile.d.ts
export declare function compile(): void
// @Filename: /package.json
{ "dependencies": { "mdx": "*" } }
// @Filename: /index.ts
mdx/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}
