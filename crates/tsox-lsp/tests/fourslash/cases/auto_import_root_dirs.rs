use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn auto_import_root_dirs() {
    let content = r#"// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "module": "commonjs",
        "rootDirs": [".", "./some/other/root"]
    }
}
// @Filename: /some/other/root/types.ts
export type Something = {};
// @Filename: /index.ts
const s: Something/**/"#;
    let mut s = Session::new_for_test("autoImportRootDirs", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"./types"}, nil /*preferences*/)
}
