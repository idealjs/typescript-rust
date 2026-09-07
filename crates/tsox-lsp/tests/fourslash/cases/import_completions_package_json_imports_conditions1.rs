use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn import_completions_package_json_imports_conditions1() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#thing": {
        "types": { "import": "./types-esm/thing.d.mts", "require": "./types/thing.d.ts" },
        "default": { "import": "./esm/thing.mjs", "require": "./dist/thing.js" }
     }
  }
}
// @Filename: /types/thing.d.ts
export function something(name: string): any;
// @Filename: /src/foo.ts
import {} from "/*1*/";"##;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
