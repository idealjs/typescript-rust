use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_paths_aliases_and_barrels() {
    let content = r#"// @Filename: /tsconfig.json
 {
  "compilerOptions": {
    "module": "commonjs",
    "paths": {
      "~/*": ["src/*"]  
    }
  }
}
// @Filename: /src/dirA/index.ts
 export * from "./thing1A";
 export * from "./thing2A";
// @Filename: /src/dirA/thing1A.ts
 export class Thing1A {}
 Thing/**/
// @Filename: /src/dirA/thing2A.ts
 export class Thing2A {}
// @Filename: /src/dirB/index.ts
 export * from "./thing1B";
 export * from "./thing2B";
// @Filename: /src/dirB/thing1B.ts
 export class Thing1B {}
// @Filename: /src/dirB/thing2B.ts
 export class Thing2B {}"#;
    let mut s = Session::new_for_test("autoImportPathsAliasesAndBarrels", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
