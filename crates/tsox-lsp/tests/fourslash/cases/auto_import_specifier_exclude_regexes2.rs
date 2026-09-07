use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"./utils"}, nil /*preferences*/)
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"@app/utils"}, &lsutil.UserPreferences{AutoImportS
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"@app/utils"}, &lsutil.UserPreferences{ImportModul
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"./utils"}, &lsutil.UserPreferences{ImportModuleSp
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{}, &lsutil.UserPreferences{AutoImportSpecifierExcl
}
