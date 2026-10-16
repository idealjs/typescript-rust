use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_default_export1() {
    let content = r#"// @Filename: /foo-bar.ts
export default function fooBar();
// @Filename: /b.ts
[|import * as fb from "./foo-bar";
foo/**/Bar|]"#;
    let mut s = Session::new_for_test("importNameCodeFixDefaultExport1", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
