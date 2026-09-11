use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_new_import_export_equals_es_next_interop_off() {
    let content = r#"// @Module: esnext
// @Filename: /foo.d.ts
declare module "foo" {
  const foo: number;
  export = foo;
}
// @Filename: /index.ts
foo"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportExportEqualsESNextInteropOff", content);
    fourslash::go_to_file(&mut s, "/index.ts");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
