use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_optional_import0() {
    let content = r#"// @Filename: a/f1.ts
[|import * as ns from "./foo";
foo/*0*/();|]
// @Filename: a/foo/bar.ts
export function foo() {};
// @Filename: a/foo.ts
export { foo } from "./foo/bar";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
