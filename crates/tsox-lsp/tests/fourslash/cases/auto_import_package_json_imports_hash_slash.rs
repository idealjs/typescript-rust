use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_package_json_imports_hash_slash_nodenext() {
    let content = r##"// @Filename: /tsconfig.json
{
  "compilerOptions": {
    "module": "nodenext",
    "rootDir": "./",
    "outDir": "build"
  }
}
// @Filename: /package.json
{
  "imports": {
    "#/*": {
      "types": "./src/*",
      "default": "./src/*"
    }
  }
}
// @Filename: /src/domain/entities/entity.ts
export const entity = 1;
// @Filename: /src/features/deep/consumer.ts
entit/**/"##;
    let mut s = Session::new_for_test("autoImportPackageJsonImportsHashSlashNodenext", content);
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}

#[test]
fn auto_import_package_json_imports_hash_slash_node16() {
    let content = r##"// @Filename: /tsconfig.json
{
  "compilerOptions": {
    "module": "node16"
  }
}
// @Filename: /package.json
{
  "imports": {
    "#/*": "./src/*"
  }
}
// @Filename: /src/domain/entities/entity.ts
export const entity = 1;
// @Filename: /src/consumer.ts
entit/**/"##;
    let mut s = Session::new_for_test("autoImportPackageJsonImportsHashSlashNode16", content);
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}
