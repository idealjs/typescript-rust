use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_import_package_json_imports_preference() {
    let content = r##"// @module: preserve
// @allowImportingTsExtensions: true
// @Filename: /project/package.json
{
  "name": "project",
  "version": "1.0.0",
  "imports": {
    "#internal/*": "./src/internal/*.ts"
  }
}
// @Filename: /project/src/internal/foo.ts
export const internalFoo = 0;
// @Filename: /project/src/other.ts
export * from "./internal/foo.ts";
// @Filename: /project/src/main.ts
internalFoo/**/"##;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
