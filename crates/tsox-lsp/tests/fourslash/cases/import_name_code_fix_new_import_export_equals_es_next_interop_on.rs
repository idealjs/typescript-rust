use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_export_equals_es_next_interop_on() {
    let content = r#"// @EsModuleInterop: true
// @Module: es2015
// @Filename: /foo.d.ts
declare module "foo" {
  const foo: number;
  export = foo;
}
// @Filename: /index.ts
[|foo|]"#;
    let mut s = Session::new_for_test("importNameCodeFixNewImportExportEqualsESNextInteropOn", content);
    fourslash::go_to_file(&mut s, "/index.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
