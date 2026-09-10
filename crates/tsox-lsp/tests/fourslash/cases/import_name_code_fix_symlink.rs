use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_symlink() {
    let content = r#"// @moduleResolution: bundler
// @noLib: true
// @Filename: /node_modules/real/index.d.ts
// @Symlink: /node_modules/link/index.d.ts
export const foo: number;
// @Filename: /a.ts
import { foo } from "link";
// @Filename: /b.ts
[|foo;|]"#;
    let mut s = Session::new_for_test("importNameCodeFix_symlink", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
