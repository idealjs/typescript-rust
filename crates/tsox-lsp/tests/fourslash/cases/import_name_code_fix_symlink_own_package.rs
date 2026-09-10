use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_symlink_own_package() {
    let content = r#"// @Filename: /packages/b/b0.ts
// @Symlink: /node_modules/b/b0.ts
x;
// @Filename: /packages/b/b1.ts
// @Symlink: /node_modules/b/b1.ts
import { a } from "a";
export const x = 0;
// @Filename: /packages/a/index.d.ts
// @Symlink: /node_modules/a/index.d.ts
export const a: number;"#;
    let mut s = Session::new_for_test("importNameCodeFix_symlink_own_package", content);
    fourslash::go_to_file(&mut s, "/packages/b/b0.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
