use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_specifier_exclude_regexes3() {
    let content = r#"// @module: preserve
// @Filename: /node_modules/pkg/package.json
{
    "name": "pkg",
    "version": "1.0.0",
    "exports": {
        ".": "./index.js",
        "./utils": "./utils.js"
    }
}
// @Filename: /node_modules/pkg/utils.d.ts
export function add(a: number, b: number) {}
// @Filename: /node_modules/pkg/index.d.ts
export * from "./utils";
// @Filename: /src/index.ts
add/**/"#;
    let mut s = Session::new_for_test("autoImportSpecifierExcludeRegexes3", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"pkg", "pkg/utils"}, nil /*preferences*/)
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"pkg/utils"}, &lsutil.UserPreferences{AutoImportSp
}
