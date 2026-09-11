use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_paths_with_extension() {
    let content = r##"// @Filename: /tsconfig.json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "Node16",
    "moduleResolution": "Node16",
    "rootDir": "./src",
    "outDir": "./dist",
    "paths": {
      "#internals/*": ["./src/internals/*.ts"]
    }
  },
  "include": ["src"]
}
// @Filename: /src/internals/example.ts
export function helloWorld() {}
// @Filename: /src/index.ts
helloWorld/**/"##;
    let mut s = Session::new_for_test("importNameCodeFix_pathsWithExtension", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"#internals/example"}, &lsutil.UserPreferences{Imp
}
