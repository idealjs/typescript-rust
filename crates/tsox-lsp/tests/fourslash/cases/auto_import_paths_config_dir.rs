use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_paths_config_dir() {
    let content = r#"// @Filename: tsconfig.json
{
    "compilerOptions": {
        "paths": {
            "@root/*": ["${configDir}/src/*"]
        }
    }
}
// @Filename: src/one.ts
export const one = 1;
// @Filename: src/foo/two.ts
one/**/"#;
    let mut s = Session::new_for_test("autoImportPathsConfigDir", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"@root/one"}, nil /*preferences*/)
}
