use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyRename"]
#[test]
fn rename_file_package_json() {
    let content = r#"// @Filename: /src/example.ts
import brushPackageJson from './visx-brush//*rename*/package.json';
// @Filename: /src/visx-brush/package.json
{ "name": "brush" }"#;
    let mut s = Session::new_for_test("renameFilePackageJson", content);
    fourslash::unsupported("VerifyRename"); // f.VerifyRename(t, "rename", "package2.json", map[string]string{
}
