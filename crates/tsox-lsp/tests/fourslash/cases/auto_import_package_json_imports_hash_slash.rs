use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
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
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}

#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
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
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}
