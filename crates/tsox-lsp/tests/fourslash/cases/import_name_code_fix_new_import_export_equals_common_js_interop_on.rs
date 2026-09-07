use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_new_import_export_equals_common_js_interop_on() {
    let content = r#"// @Module: commonjs
// @EsModuleInterop: true
// @Filename: /foo.d.ts
declare module "bar" {
  const bar: number;
  export = bar;
}
declare module "foo" {
  const foo: number;
  export = foo;
}
declare module "es" {
  const es = 0;
  export default es;
}
// @Filename: /a.ts
import bar = require("bar");

foo
// @Filename: /b.ts
foo
// @Filename: /c.ts
import es from "es";
import bar = require("bar");

foo"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/a.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/c.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
