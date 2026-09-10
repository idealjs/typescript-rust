use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_symlink_own_package_2() {
    let content = r#"// @Filename: /packages/a/test.ts
// @Symlink: /node_modules/a/test.ts
x;
// @Filename: /packages/a/utils.ts
// @Symlink: /node_modules/a/utils.ts
import {} from "a/utils";
export const x = 0;"#;
    let mut s = Session::new_for_test("importNameCodeFix_symlink_own_package_2", content);
    fourslash::go_to_file(&mut s, "/packages/a/test.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
