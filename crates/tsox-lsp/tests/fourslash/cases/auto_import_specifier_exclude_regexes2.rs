use tsox_lsp::fourslash::Session;


#[test]
fn auto_import_specifier_exclude_regexes2() {
    let content = r#"// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "module": "preserve",
        "paths": {
            "@app/*": ["./src/*"]
        }
    }
}
// @Filename: /src/utils.ts
export function add(a: number, b: number) {}
// @Filename: /src/index.ts
add/**/"#;
    let _s = Session::new_for_test("autoImportSpecifierExcludeRegexes2", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"./utils"}, nil /*preferences*/)
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"@app/utils"}, &lsutil.UserPreferences{AutoImportS
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"@app/utils"}, &lsutil.UserPreferences{ImportModul
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"./utils"}, &lsutil.UserPreferences{ImportModuleSp
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{}, &lsutil.UserPreferences{AutoImportSpecifierExcl
}
